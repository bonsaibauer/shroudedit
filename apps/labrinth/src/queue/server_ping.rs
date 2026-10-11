use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{TimeDelta, Utc};
use clickhouse::{Client, Row};
use serde::Serialize;
use sqlx::types::Json;
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::Semaphore,
    task::JoinSet,
};
use tracing::{info, warn};
use xredis::RedisPool;

use crate::{
    database::{DBProject, PgPool, models::DBProjectId},
    env::ENV,
    models::{exp, ids::ProjectId, projects::ProjectStatus},
    search::incremental::IncrementalSearchQueue,
    util::error::Context,
};

pub const ENSHROUDED_REDIS_NAMESPACE: &str = "enshrouded_server_ping:v1";
pub const ENSHROUDED_REDIS_FAILURE_NAMESPACE: &str =
    "enshrouded_server_ping_failures:v1";
pub const CLICKHOUSE_TABLE: &str = "enshrouded_server_pings";

pub struct ServerPingQueue {
    pub db: PgPool,
    pub redis: RedisPool,
    pub clickhouse: Client,
    pub incremental_search_queue: IncrementalSearchQueue,
}

impl ServerPingQueue {
    pub fn new(
        db: PgPool,
        redis: RedisPool,
        clickhouse: Client,
        incremental_search_queue: IncrementalSearchQueue,
    ) -> Self {
        Self {
            db,
            redis,
            clickhouse,
            incremental_search_queue,
        }
    }

    pub async fn ping_enshrouded_servers(&self) -> eyre::Result<()> {
        let servers = self.find_enshrouded_servers_to_ping().await?;
        info!(count = servers.len(), "found Enshrouded servers to ping");
        let active_pings =
            Arc::new(Semaphore::new(ENV.SERVER_PING_MAX_CONCURRENT));
        let pings = servers
            .into_iter()
            .map(|(project_id, server)| {
                let active_pings = active_pings.clone();
                tokio::spawn(async move {
                    let _permit = active_pings
                        .acquire()
                        .await
                        .expect("semaphore should not be closed");
                    let data = ping_enshrouded_server(
                        (server.address.as_str(), server.query_port),
                        None,
                    )
                    .await
                    .ok();
                    (
                        project_id,
                        exp::enshrouded::EnshroudedServerPing {
                            when: Utc::now(),
                            address: server.address,
                            query_port: server.query_port,
                            data,
                        },
                    )
                })
            })
            .collect::<JoinSet<_>>()
            .join_all()
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if !pings.is_empty() {
            let mut clickhouse = self
                .clickhouse
                .insert::<EnshroudedServerPingRecord>(CLICKHOUSE_TABLE)
                .await
                .wrap_err(
                    "failed to begin inserting Enshrouded ping records",
                )?;
            for (project_id, ping) in &pings {
                let data = ping.data.as_ref();
                clickhouse
                    .write(&EnshroudedServerPingRecord {
                        recorded: ping
                            .when
                            .timestamp_nanos_opt()
                            .unwrap_or_default()
                            / 100_000,
                        project_id: project_id.0,
                        address: ping.address.clone(),
                        query_port: ping.query_port,
                        online: data.is_some(),
                        latency_ms: data
                            .map(|data| data.latency.as_millis() as u32),
                        name: data.map(|data| data.name.clone()),
                        game_version: data
                            .map(|data| data.game_version.clone()),
                        map: data.map(|data| data.map.clone()),
                        players_online: data.map(|data| data.players_online),
                        players_max: data.map(|data| data.players_max),
                        password_protected: data
                            .map(|data| data.password_protected),
                    })
                    .await
                    .wrap_err("failed to write Enshrouded ping record")?;
            }
            clickhouse.end().await.wrap_err(
                "failed to finish inserting Enshrouded ping records",
            )?;
        }

        let mut redis = self
            .redis
            .connect()
            .await
            .wrap_err("failed to connect to redis")?;
        for (project_id, ping) in &pings {
            let ping_key = self
                .redis
                .key()
                .entity(ENSHROUDED_REDIS_NAMESPACE, project_id);
            let failure_key = self
                .redis
                .key()
                .entity(ENSHROUDED_REDIS_FAILURE_NAMESPACE, project_id);
            let updated = if ping.data.is_some() {
                redis
                    .set_serialized(&ping_key, ping, None)
                    .await
                    .wrap_err("failed to cache Enshrouded server ping")?;
                redis.delete(&failure_key).await?;
                true
            } else {
                let failures =
                    redis.incr(&failure_key).await?.unwrap_or_default();
                if failures >= ENV.SERVER_PING_MAX_FAIL_COUNT {
                    redis
                        .set_serialized(&ping_key, ping, None)
                        .await
                        .wrap_err(
                            "failed to cache failed Enshrouded server ping",
                        )?;
                    true
                } else {
                    false
                }
            };
            if updated {
                DBProject::clear_cache(
                    (*project_id).into(),
                    None,
                    None,
                    &self.redis,
                )
                .await
                .inspect_err(|error| {
                    warn!(?error, "failed to clear Enshrouded server cache")
                })
                .ok();
                self.incremental_search_queue
                    .push_project_change(*project_id)
                    .await;
            }
        }
        Ok(())
    }

    async fn find_enshrouded_servers_to_ping(
        &self,
    ) -> eyre::Result<Vec<(ProjectId, exp::enshrouded::EnshroudedServerProject)>>
    {
        let rows = sqlx::query!(
			r#"SELECT id, components AS "components: Json<exp::ProjectSerial>" FROM mods WHERE status = ANY($1) AND components ? 'enshrouded_server'"#,
			&ProjectStatus::iterator().filter(|status| status.is_approved()).map(|status| status.to_string()).collect::<Vec<_>>()
		).fetch_all(&self.db).await.wrap_err("failed to fetch Enshrouded servers")?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let project_ids = rows
            .iter()
            .map(|row| ProjectId::from(DBProjectId(row.id)))
            .collect::<Vec<_>>();
        let ping_keys = project_ids
            .iter()
            .map(|id| self.redis.key().entity(ENSHROUDED_REDIS_NAMESPACE, id))
            .collect::<Vec<_>>();
        let mut redis = self.redis.connect().await?;
        let previous = redis
            .get_many_deserialized::<exp::enshrouded::EnshroudedServerPing>(
                &ping_keys,
            )
            .await?;
        let now = Utc::now();
        Ok(rows
            .into_iter()
            .zip(previous)
            .filter(|(row, ping)| {
                let Some(server) = &row.components.0.enshrouded_server else {
                    return false;
                };
                !server.address.trim().is_empty()
                    && ping.as_ref().is_none_or(|ping| {
                        ping.data.is_none()
                            || now.signed_duration_since(ping.when)
                                > TimeDelta::seconds(
                                    ENV.SERVER_PING_MIN_INTERVAL_SEC as i64,
                                )
                    })
            })
            .filter_map(|(row, _)| {
                Some((
                    ProjectId::from(DBProjectId(row.id)),
                    row.components.0.enshrouded_server?,
                ))
            })
            .collect())
    }
}

#[derive(Debug, Row, Serialize)]
struct EnshroudedServerPingRecord {
    recorded: i64,
    project_id: u64,
    address: String,
    query_port: u16,
    online: bool,
    latency_ms: Option<u32>,
    name: Option<String>,
    game_version: Option<String>,
    map: Option<String>,
    players_online: Option<u8>,
    players_max: Option<u8>,
    password_protected: Option<bool>,
}

pub async fn ping_enshrouded_server(
    address: impl ToSocketAddrs,
    timeout: Option<Duration>,
) -> eyre::Result<exp::enshrouded::EnshroudedServerPingData> {
    let start = Instant::now();
    let maximum = Duration::from_millis(ENV.SERVER_PING_TIMEOUT_MS);
    let timeout = timeout
        .map(|duration| duration.min(maximum))
        .unwrap_or(maximum);
    let task = async move {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .wrap_err("failed to bind UDP socket")?;
        socket
            .connect(address)
            .await
            .wrap_err("failed to resolve or connect to Enshrouded server")?;
        let mut request = b"\xFF\xFF\xFF\xFFTSource Engine Query\0".to_vec();
        let mut response = send_a2s_info_request(&socket, &request).await?;
        if response.get(4) == Some(&0x41) {
            request.extend_from_slice(
                response
                    .get(5..9)
                    .wrap_err("invalid A2S challenge response")?,
            );
            response = send_a2s_info_request(&socket, &request).await?;
        }
        parse_a2s_info(&response, start.elapsed())
    };
    tokio::time::timeout(timeout, task)
        .await
        .map_err(eyre::Error::new)
        .flatten()
}

async fn send_a2s_info_request(
    socket: &UdpSocket,
    request: &[u8],
) -> eyre::Result<Vec<u8>> {
    socket
        .send(request)
        .await
        .wrap_err("failed to send A2S_INFO request")?;
    let mut buffer = vec![0_u8; 1400];
    let length = socket
        .recv(&mut buffer)
        .await
        .wrap_err("failed to receive A2S_INFO response")?;
    buffer.truncate(length);
    Ok(buffer)
}

fn parse_a2s_info(
    response: &[u8],
    latency: Duration,
) -> eyre::Result<exp::enshrouded::EnshroudedServerPingData> {
    eyre::ensure!(
        response.starts_with(&[0xff, 0xff, 0xff, 0xff, 0x49]),
        "invalid A2S_INFO response"
    );
    let mut cursor = 6;
    let name = read_a2s_string(response, &mut cursor)?;
    let map = read_a2s_string(response, &mut cursor)?;
    let _folder = read_a2s_string(response, &mut cursor)?;
    let _game = read_a2s_string(response, &mut cursor)?;
    cursor += 2;
    let players_online =
        *response.get(cursor).wrap_err("missing A2S player count")?;
    let players_max = *response
        .get(cursor + 1)
        .wrap_err("missing A2S maximum player count")?;
    let password_protected = response
        .get(cursor + 5)
        .copied()
        .wrap_err("missing A2S visibility flag")?
        != 0;
    cursor += 7;
    let game_version = read_a2s_string(response, &mut cursor)?;
    Ok(exp::enshrouded::EnshroudedServerPingData {
        latency,
        name,
        game_version,
        map,
        players_online,
        players_max,
        password_protected,
    })
}

fn read_a2s_string(
    response: &[u8],
    cursor: &mut usize,
) -> eyre::Result<String> {
    let remaining = response
        .get(*cursor..)
        .wrap_err("invalid A2S_INFO string offset")?;
    let length = remaining
        .iter()
        .position(|byte| *byte == 0)
        .wrap_err("unterminated A2S_INFO string")?;
    let value = String::from_utf8_lossy(&remaining[..length]).into_owned();
    *cursor += length + 1;
    Ok(value)
}
