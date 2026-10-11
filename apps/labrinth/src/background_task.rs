use crate::database;
use crate::database::PgPool;
use crate::database::models::ids::DBUserId;
use crate::database::models::notification_item::NotificationBuilder;
use crate::file_hosting::FileHost;
use crate::models::notifications::NotificationBody;
use crate::queue::analytics::cache::cache_analytics;
use crate::queue::billing::{index_billing, index_subscriptions};
use crate::queue::email::EmailQueue;
use crate::queue::file_scan::scan_all_pending_files;
use crate::queue::payouts::{
    PayoutsQueue, index_payouts_notifications,
    insert_bank_balances_and_webhook, process_affiliate_payouts,
    process_payout, remove_payouts_for_refunded_charges,
};
use crate::search::SearchBackend;
use crate::util::anrok;
use actix_web::web;
use clap::ValueEnum;
use eyre::WrapErr;
use tracing::{info, instrument};
use xredis::RedisPool;

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum BackgroundTask {
    IndexSearch,
    ReleaseScheduled,
    Payouts,
    SyncPayoutStatuses,
    IndexBilling,
    IndexSubscriptions,
    IncrementalIndexSearch,
    DelphiFileScan,
    Migrations,
    Mail,
    /// Queries server project analytics (e.g. number of verified plays in last
    /// 2 weeks for server projects) and caches them in Redis.
    CacheAnalytics,
    /// Queries Enshrouded dedicated servers for status and player information.
    PingEnshroudedServers,
    /// Finds files of versions which have not been scanned for attributions
    /// yet, extracts them to find file overrides, and finds any overrides which
    /// require attribution from the creator.
    ScanPendingFiles,
    /// Queues Discord Creator Club role claim emails for newly eligible users.
    DiscordRoleEmailCampaign,
}

impl BackgroundTask {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all, fields(background_task = ?self))]
    pub async fn run(
        self,
        pool: PgPool,
        ro_pool: PgPool,
        redis_pool: RedisPool,
        search_backend: web::Data<dyn SearchBackend>,
        file_host: web::Data<dyn FileHost>,
        kafka_client: web::Data<crate::util::kafka::KafkaClientState>,
        clickhouse: clickhouse::Client,
        stripe_client: stripe::Client,
        anrok_client: anrok::Client,
        email_queue: EmailQueue,
        mural_client: muralpay::Client,
    ) -> eyre::Result<()> {
        use BackgroundTask::*;
        match self {
            Migrations => run_migrations().await,
            IndexSearch => {
                index_search(ro_pool, redis_pool, search_backend).await
            }
            ReleaseScheduled => release_scheduled(pool).await,
            Payouts if crate::env::ENV.MONETIZATION_ENABLED => {
                payouts(pool, clickhouse, redis_pool).await
            }
            Payouts => Ok(()),
            SyncPayoutStatuses if crate::env::ENV.MONETIZATION_ENABLED => {
                sync_payout_statuses(pool, mural_client).await
            }
            SyncPayoutStatuses => Ok(()),
            IndexBilling if crate::env::ENV.MONETIZATION_ENABLED => {
                index_billing(
                    stripe_client,
                    anrok_client,
                    pool.clone(),
                    redis_pool,
                )
                .await;

                update_bank_balances(pool).await
            }
            IndexBilling => Ok(()),
            IndexSubscriptions if crate::env::ENV.MONETIZATION_ENABLED => {
                index_subscriptions(
                    pool,
                    redis_pool,
                    stripe_client,
                    anrok_client,
                )
                .await;
                Ok(())
            }
            IndexSubscriptions => Ok(()),
            IncrementalIndexSearch => {
                crate::search::incremental::consume::run(
                    ro_pool,
                    redis_pool,
                    search_backend,
                    kafka_client,
                )
                .await
            }
            DelphiFileScan => {
                crate::queue::delphi_scan::run(pool, kafka_client).await
            }
            Mail => run_email(email_queue).await,
            CacheAnalytics => {
                cache_analytics(&pool, &redis_pool, &clickhouse).await
            }
            PingEnshroudedServers => {
                ping_enshrouded_servers(
                    pool,
                    redis_pool,
                    clickhouse,
                    kafka_client,
                )
                .await
            }
            ScanPendingFiles => {
                scan_all_pending_files(
                    &pool,
                    &redis_pool,
                    file_host.into_inner(),
                )
                .await
            }
            DiscordRoleEmailCampaign => {
                discord_role_email_campaign(pool, redis_pool).await
            }
        }
    }
}

pub async fn run_email(email_queue: EmailQueue) -> eyre::Result<()> {
    // Only index for 5 emails at a time, to reduce transaction length,
    // for a total of 100 emails.
    for _ in 0..20 {
        let then = std::time::Instant::now();

        let indexed = email_queue
            .index(5)
            .await
            .wrap_err("failed to index email queue")?;
        if indexed {
            info!("Indexed email queue in {}ms", then.elapsed().as_millis());
        } else {
            info!("No more emails to index");
            break;
        }
    }

    Ok(())
}

pub async fn update_bank_balances(pool: PgPool) -> eyre::Result<()> {
    let payouts_queue = PayoutsQueue::new();

    insert_bank_balances_and_webhook(&payouts_queue, &pool)
        .await
        .wrap_err("failed to update bank balances")?;
    info!("Bank balances updated successfully");
    Ok(())
}

pub async fn run_migrations() -> eyre::Result<()> {
    database::check_for_migrations().await?;
    crate::clickhouse::run_migrations()
        .await
        .wrap_err("failed to run ClickHouse migrations")?;
    Ok(())
}

pub async fn index_search(
    ro_pool: PgPool,
    redis_pool: RedisPool,
    search_backend: web::Data<dyn SearchBackend>,
) -> eyre::Result<()> {
    info!("Indexing local database");
    search_backend.rebuild_index(ro_pool, redis_pool).await
}

pub async fn release_scheduled(pool: PgPool) -> eyre::Result<()> {
    info!("Releasing scheduled versions/projects!");

    sqlx::query!(
        "
        UPDATE mods
        SET status = requested_status
        WHERE status = $1 AND approved < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::ProjectStatus::Scheduled.as_str(),
    )
    .execute(&pool)
    .await
    .wrap_err("failed syncing scheduled releases for projects")?;

    sqlx::query!(
        "
        UPDATE versions
        SET status = requested_status
        WHERE status = $1 AND date_published < CURRENT_DATE AND requested_status IS NOT NULL
        ",
        crate::models::projects::VersionStatus::Scheduled.as_str(),
    )
    .execute(&pool)
    .await
    .wrap_err("failed syncing scheduled releases for versions")?;

    info!("Finished releasing scheduled versions/projects");
    Ok(())
}

pub async fn payouts(
    pool: PgPool,
    clickhouse: clickhouse::Client,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    if !crate::env::ENV.MONETIZATION_ENABLED {
        return Ok(());
    }

    info!("Started running payouts");
    process_payout(&pool, &clickhouse)
        .await
        .wrap_err("payout processing failed")?;

    index_payouts_notifications(&pool, &redis_pool)
        .await
        .wrap_err("payout notifications indexing failed")?;

    process_affiliate_payouts(&pool)
        .await
        .wrap_err("affiliate payouts processing failed")?;

    remove_payouts_for_refunded_charges(&pool)
        .await
        .wrap_err("removing payouts for refunded charges failed")?;

    info!("Done running payouts");
    Ok(())
}

pub async fn discord_role_email_campaign(
    pool: PgPool,
    redis_pool: RedisPool,
) -> eyre::Result<()> {
    info!("Started indexing Discord role email campaign");

    let mut txn = pool
        .begin()
        .await
        .wrap_err("failed to begin Discord role email campaign transaction")?;

    let lock_acquired =
        crate::database::advisory_lock::AdvisoryLock::DiscordRoleEmailCampaign
            .try_acquire(&mut txn)
            .await
            .wrap_err("failed to acquire Discord role email campaign lock")?;

    if !lock_acquired {
        info!("Discord role email campaign is already running");
        return Ok(());
    }

    let user_ids = sqlx::query_scalar!(
        r#"
        WITH
          user_project_downloads AS (
            SELECT
              tm.user_id,
              SUM(m.downloads)::BIGINT total_downloads
            FROM team_members tm
            INNER JOIN mods m ON m.team_id = tm.team_id
            WHERE tm.accepted = TRUE
            GROUP BY tm.user_id
          )
        SELECT u.id AS "id!"
        FROM users u
        INNER JOIN user_project_downloads upd ON upd.user_id = u.id
        WHERE u.email IS NOT NULL
          AND u.email_verified = TRUE
          AND upd.total_downloads > 20000
          AND NOT EXISTS (
            SELECT 1
            FROM notifications n
            WHERE n.user_id = u.id
              AND n.body ->> 'type' = 'discord_role_creator_club'
          )
        ORDER BY upd.total_downloads DESC, u.id
        LIMIT 1000
        "#,
    )
    .fetch_all(&mut txn)
    .await
    .wrap_err("failed to fetch Discord role email campaign recipients")?
    .into_iter()
    .map(DBUserId)
    .collect::<Vec<_>>();

    let count = user_ids.len();

    if !user_ids.is_empty() {
        NotificationBuilder {
            body: NotificationBody::DiscordRoleCreatorClub,
        }
        .insert_many(user_ids, &mut txn, &redis_pool)
        .await
        .wrap_err("failed to queue Discord role email notifications")?;
    }

    txn.commit()
        .await
        .wrap_err("failed to commit Discord role email campaign transaction")?;

    info!(count, "Finished indexing Discord role email campaign");
    Ok(())
}

pub async fn sync_payout_statuses(
    pool: PgPool,
    mural: muralpay::Client,
) -> eyre::Result<()> {
    // Mural sets a max limit of 100 for search payouts endpoint
    const LIMIT: u32 = 100;

    info!("Started syncing payout statuses");

    crate::queue::payouts::mural::sync_pending_payouts_from_mural(
        &pool, &mural, LIMIT,
    )
    .await
    .wrap_err("failed to sync pending payouts from Mural")?;

    crate::queue::payouts::mural::sync_failed_mural_payouts_to_labrinth(
        &pool, &mural, LIMIT,
    )
    .await
    .wrap_err("failed to sync failed Mural payouts to Labrinth")?;

    info!("Done syncing payout statuses");
    Ok(())
}

pub async fn ping_enshrouded_servers(
    pool: PgPool,
    redis_pool: RedisPool,
    clickhouse: clickhouse::Client,
    kafka_client: web::Data<crate::util::kafka::KafkaClientState>,
) -> eyre::Result<()> {
    info!("Started pinging Enshrouded servers");

    let incremental_search_queue =
        crate::search::incremental::IncrementalSearchQueue::new(kafka_client);
    let server_ping_queue = crate::queue::server_ping::ServerPingQueue::new(
        pool,
        redis_pool,
        clickhouse,
        incremental_search_queue.clone(),
    );

    server_ping_queue
        .ping_enshrouded_servers()
        .await
        .wrap_err("failed to ping Enshrouded servers")?;
    incremental_search_queue
        .drain()
        .await
        .wrap_err("failed to drain incremental search queue")?;
    info!("Done pinging Enshrouded servers");
    Ok(())
}
