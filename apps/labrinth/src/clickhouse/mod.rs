use std::time::Duration;

use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::rt::TokioExecutor;

mod fetch;

pub use fetch::*;

use crate::env::ENV;
use crate::queue::server_ping;
use crate::routes::analytics::ENSHROUDED_SERVER_PLAYS;

pub const DOWNLOADS: &str = "downloads";
pub const PLAYTIME: &str = "playtime";

pub async fn init_client() -> clickhouse::error::Result<clickhouse::Client> {
    init_client_with_database(&ENV.CLICKHOUSE_DATABASE).await
}

pub async fn init_client_with_database(
    database: &str,
) -> clickhouse::error::Result<clickhouse::Client> {
    Ok(connect()?.with_database(database))
}

fn connect() -> clickhouse::error::Result<clickhouse::Client> {
    let https_connector = HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_all_versions()
        .build();
    let hyper_client =
        hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            // ClickHouse closes idle keep-alive connections after 3s by default.
            // Hyper's 90s default reuses dead sockets and fails with
            // "connection closed before message completed".
            .pool_idle_timeout(Duration::from_secs(2))
            .build(https_connector);

    Ok(clickhouse::Client::with_http_client(hyper_client)
        .with_url(&ENV.CLICKHOUSE_URL)
        .with_user(&ENV.CLICKHOUSE_USER)
        .with_password(&ENV.CLICKHOUSE_PASSWORD)
        .with_validation(false))
}

#[cfg(feature = "test")]
pub async fn create_database(database: &str) -> clickhouse::error::Result<()> {
    connect()?
        .query(&format!("CREATE DATABASE IF NOT EXISTS {database}"))
        .execute()
        .await
}

pub async fn run_migrations() -> clickhouse::error::Result<()> {
    run_migrations_on_database(&ENV.CLICKHOUSE_DATABASE).await
}

pub async fn run_migrations_on_database(
    database: &str,
) -> clickhouse::error::Result<()> {
    const ENSHROUDED_SERVER_PINGS: &str = server_ping::CLICKHOUSE_TABLE;

    let client = connect()?;

    let clickhouse_replicated = ENV.CLICKHOUSE_REPLICATED;
    let cluster_line = if clickhouse_replicated {
        "ON cluster '{cluster}'"
    } else {
        ""
    };

    let engine = if clickhouse_replicated {
        "ReplicatedMergeTree('/clickhouse/{installation}/{cluster}/tables/{shard}/{database}/{table}', '{replica}')"
    } else {
        "MergeTree()"
    };

    // For the Clickhouse database on the staging environment, set a TTL to avoid accumulating too much data
    let ttl = if database == "staging_analytics" {
        "TTL toDateTime(recorded) + INTERVAL 1 DAY"
    } else {
        ""
    };

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.views {cluster_line}
            (
                recorded DateTime64(4),
                domain String,
                site_path String,

                user_id UInt64,
                project_id UInt64,
                monetized Bool DEFAULT True,

                ip IPv6,
                country String,
                user_agent String,
                headers Array(Tuple(String, String)) DEFAULT [] TTL toDateTime(recorded) + INTERVAL 60 DAY
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{DOWNLOADS} {cluster_line}
            (
                recorded DateTime64(4),
                domain String,
                site_path String,

                user_id UInt64,
                project_id UInt64,
                version_id UInt64,

                ip IPv6,
                country String,
                user_agent String,
                headers Array(Tuple(String, String)) DEFAULT [] TTL toDateTime(recorded) + INTERVAL 60 DAY
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, ip)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{PLAYTIME} {cluster_line}
            (
                recorded DateTime64(4),
                seconds UInt64,

                user_id UInt64,
                project_id UInt64,
                version_id UInt64,

                loader String,
                game_version String,
                parent UInt64
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded, user_id)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.affiliate_code_clicks {cluster_line}
            (
                recorded DateTime64(4),
                domain String,

                user_id UInt64,
                affiliate_code_id UInt64,

                ip IPv6,
                country String,
                user_agent String,
                headers Array(Tuple(String, String)) DEFAULT [] TTL toDateTime(recorded) + INTERVAL 60 DAY
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (affiliate_code_id, recorded)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{ENSHROUDED_SERVER_PINGS} {cluster_line}
            (
                recorded DateTime64(4),
                project_id UInt64,
                address String,
                query_port UInt16,
                online Bool,
                latency_ms Nullable(UInt32),
                name Nullable(String),
                game_version Nullable(String),
                map Nullable(String),
                players_online Nullable(UInt8),
                players_max Nullable(UInt8),
                password_protected Nullable(Bool)
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {database}.{ENSHROUDED_SERVER_PLAYS} {cluster_line}
            (
                recorded DateTime64(4),
                user_id UInt64,
                project_id UInt64,
                enshrouded_uuid UUID
            )
            ENGINE = {engine}
            PARTITION BY toYYYYMM(recorded)
            {ttl}
            PRIMARY KEY (project_id, recorded)
            SETTINGS index_granularity = 8192
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{ENSHROUDED_SERVER_PLAYS} {cluster_line}
            ADD COLUMN IF NOT EXISTS enshrouded_uuid UUID
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{ENSHROUDED_SERVER_PLAYS} {cluster_line}
            ADD COLUMN IF NOT EXISTS ip IPv6 DEFAULT toIPv6('::')
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{DOWNLOADS} {cluster_line}
            ADD COLUMN IF NOT EXISTS reason String,
            ADD COLUMN IF NOT EXISTS game_version String,
            ADD COLUMN IF NOT EXISTS loader String,
            ADD COLUMN IF NOT EXISTS dependent_on_version_id UInt64
            "
        ))
        .execute()
        .await?;

    client
        .query(&format!(
            "
            ALTER TABLE {database}.{PLAYTIME} {cluster_line}
            ADD COLUMN IF NOT EXISTS country String
            "
        ))
        .execute()
        .await?;

    Ok(())
}
