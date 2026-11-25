use std::time::Duration;

use actix_web::web;
use chrono::Utc;
use eyre::{Result, WrapErr, eyre};
use rdkafka::{
	Message, Offset,
	consumer::{CommitMode, Consumer, StreamConsumer},
	message::BorrowedMessage,
	producer::FutureRecord,
	util::Timeout,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use uuid::Uuid;
use xredis::RedisPool;

use crate::{
	database::{
		PgPool, PgTransaction,
		advisory_lock::AdvisoryLock,
		models::{DBFileId, DBVersionId},
	},
	env::ENV,
	file_hosting::{FileHost, FileHostPublicity},
	models::ids::FileId,
	routes::internal::moderation::tech_review_queue,
	util::{
		kafka::{
			KAFKA_OPERATION_INTERVAL, KafkaClientState, KafkaEvent,
			VIRUSTOTAL_FILE_SCAN_TASK,
		},
		virustotal::VirusTotalClient,
	},
};

pub const VIRUSTOTAL_FILE_SCAN_TOPIC: &str =
	"private.labrinth.virustotal-file-scan.v1";

const MAX_DIRECT_UPLOAD_BYTES: usize = 32 * 1024 * 1024;
const MAX_SUBMISSION_ATTEMPTS: i16 = 5;
const MAX_POLL_ATTEMPTS: i16 = 5;
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VirusTotalFileScanEvent {
	pub file_id: FileId,
	pub force: bool,
}

#[derive(Debug, Deserialize)]
struct VirusTotalFileScanMessage {
	event_metadata: EventMetadata,
	file_id: FileId,
}

#[derive(Debug, Deserialize)]
struct EventMetadata {
	event_id: Uuid,
}

pub async fn enqueue_file(
	transaction: &mut PgTransaction<'_>,
	kafka_client: &KafkaClientState,
	file_id: DBFileId,
) -> Result<()> {
	enqueue_file_inner(transaction, kafka_client, file_id, false).await
}

pub async fn force_enqueue_file(
	transaction: &mut PgTransaction<'_>,
	kafka_client: &KafkaClientState,
	file_id: DBFileId,
) -> Result<()> {
	enqueue_file_inner(transaction, kafka_client, file_id, true).await
}

async fn enqueue_file_inner(
	transaction: &mut PgTransaction<'_>,
	kafka_client: &KafkaClientState,
	file_id: DBFileId,
	force: bool,
) -> Result<()> {
	AdvisoryLock::VirusTotalFile(file_id)
		.acquire(transaction)
		.await
		.wrap_err("locking file for VirusTotal scan enqueue")?;

	let event = KafkaEvent::new(
		VIRUSTOTAL_FILE_SCAN_TOPIC,
		VirusTotalFileScanEvent {
			file_id: file_id.into(),
			force,
		},
	);
	let queued = sqlx::query!(
		r#"
		INSERT INTO virustotal_file_scans (file_id, status, event_id)
		VALUES ($1, 'queued', $2)
		ON CONFLICT (file_id) DO UPDATE SET
			sha256 = NULL,
			analysis_id = NULL,
			status = 'queued',
			analysis = NULL,
			error = NULL,
			submission_attempts = 0,
			poll_attempts = 0,
			submitted_at = NULL,
			checked_at = NULL,
			next_poll_at = NULL,
			event_id = EXCLUDED.event_id,
			updated_at = NOW()
		WHERE $3 OR (virustotal_file_scans.status = 'completed' AND virustotal_file_scans.submitted_at IS NULL)
		"#,
		file_id.0,
		event.event_metadata.event_id,
		force,
	)
	.execute(&mut *transaction)
	.await
	.wrap_err("recording queued VirusTotal scan")?;
	if queued.rows_affected() == 0 {
		return Ok(());
	}
	let key = event.data.file_id.to_string();
	let payload = serde_json::to_vec(&event)
		.wrap_err("serializing VirusTotal file scan event")?;
	let record = FutureRecord::to(VIRUSTOTAL_FILE_SCAN_TOPIC)
		.key(&key)
		.payload(&payload);

	tokio::time::timeout(
		KAFKA_OPERATION_INTERVAL,
		kafka_client
			.client
			.send(record, Timeout::After(KAFKA_OPERATION_INTERVAL)),
	)
	.await
	.wrap_err("timing out VirusTotal file scan event publication")?
	.map_err(|(err, _)| eyre!(err))
	.wrap_err("publishing VirusTotal file scan event")?;

	Ok(())
}

pub async fn run(
	pool: PgPool,
	kafka_client: web::Data<KafkaClientState>,
	file_host: web::Data<dyn FileHost>,
	redis_pool: RedisPool,
) -> Result<()> {
	let client = VirusTotalClient::new()?;
	let consumer = &kafka_client.virustotal_file_scan_consumer;
	consumer
		.subscribe(&[VIRUSTOTAL_FILE_SCAN_TOPIC])
		.wrap_err("subscribing to VirusTotal file scan Kafka topic")?;

	info!(
		kafka.topic = VIRUSTOTAL_FILE_SCAN_TOPIC,
		kafka.consumer_group = VIRUSTOTAL_FILE_SCAN_TASK,
		"Started VirusTotal file scan Kafka consumer"
	);

	let mut poll_interval = tokio::time::interval(POLL_INTERVAL);
	let mut backfill_interval = tokio::time::interval(Duration::from_secs(60));
	backfill_interval
		.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
	let mut reconcile_review_queue = true;
	loop {
		tokio::select! {
			_ = backfill_interval.tick() => {
				if reconcile_review_queue {
					match tech_review_queue::reconcile_cleared_projects(&pool).await {
						Ok(()) => reconcile_review_queue = false,
						Err(err) => warn!("Reconciling technical review queue failed: {err:#}"),
					}
				}
				if let Err(err) = enqueue_unscanned_files(&pool, &kafka_client).await {
					warn!("Queueing unscanned version files failed: {err:#}");
				}
			}
			_ = poll_interval.tick() => {
				if let Err(err) = poll_submitted_analyses(&pool, &client, &redis_pool).await {
					warn!("VirusTotal analysis polling failed: {err:#}");
				}
			}
			message = consumer.recv() => {
				let message = message.wrap_err("receiving VirusTotal file scan Kafka message")?;
				if let Err(err) = process_message(
					&pool,
					consumer,
					&message,
					file_host.get_ref(),
					&client,
					&redis_pool,
				).await {
					tracing::error!(
						kafka.topic = message.topic(),
						kafka.partition = message.partition(),
						kafka.offset = message.offset(),
						"Failed to process VirusTotal file scan event: {err:?}"
					);
					consumer.seek(
						message.topic(),
						message.partition(),
						Offset::Offset(message.offset()),
						Timeout::After(KAFKA_OPERATION_INTERVAL),
					).wrap_err("seeking failed VirusTotal file scan Kafka message")?;
					tokio::time::sleep(KAFKA_OPERATION_INTERVAL).await;
				}
			}
		}
	}
}

async fn enqueue_unscanned_files(
	pool: &PgPool,
	kafka_client: &KafkaClientState,
) -> Result<()> {
	let file_ids = sqlx::query_scalar!(
		r#"
		SELECT f.id
		FROM files f
		LEFT JOIN virustotal_file_scans scan ON scan.file_id = f.id
		WHERE (scan.file_id IS NULL OR (scan.status = 'completed' AND scan.submitted_at IS NULL))
			AND (SELECT COUNT(*) FROM virustotal_file_scans WHERE status = 'queued') < 16
		ORDER BY f.id
		LIMIT 4
		"#,
	)
	.fetch_all(pool)
	.await
	.wrap_err("finding files without their own VirusTotal scan")?;
	for file_id in file_ids {
		let mut transaction = pool
			.begin()
			.await
			.wrap_err("beginning file scan backfill")?;
		enqueue_file(&mut transaction, kafka_client, DBFileId(file_id)).await?;
		transaction
			.commit()
			.await
			.wrap_err("committing file scan backfill")?;
	}
	Ok(())
}

async fn process_message(
	pool: &PgPool,
	consumer: &StreamConsumer,
	message: &BorrowedMessage<'_>,
	file_host: &dyn FileHost,
	client: &VirusTotalClient,
	redis_pool: &RedisPool,
) -> Result<()> {
	let Some(payload) = message.payload() else {
		return commit_message(consumer, message);
	};
	let event =
		match serde_json::from_slice::<VirusTotalFileScanMessage>(payload) {
			Ok(event) => event,
			Err(err) => {
				tracing::error!(
					"Skipping malformed VirusTotal file scan event: {err:?}"
				);
				return commit_message(consumer, message);
			}
		};
	let file_id = DBFileId::from(event.file_id);
	let scan_id = event.event_metadata.event_id;

	if !wait_for_file_transaction(pool, file_id).await? {
		info!(%event.file_id, %scan_id, "Discarding VirusTotal scan for missing file");
		return commit_message(consumer, message);
	}

	let file = sqlx::query!(
		r#"
		SELECT url, filename, size,
			version.mod_id AS "project_id!: crate::database::models::DBProjectId",
			version.id AS "version_id!: DBVersionId"
		FROM files
		INNER JOIN versions version ON version.id = files.version_id
		WHERE files.id = $1
		"#,
		file_id.0,
	)
	.fetch_optional(pool)
	.await
	.wrap_err("fetching VirusTotal scan file")?
	.ok_or_else(|| eyre!("VirusTotal scan file no longer exists"))?;

	let existing = sqlx::query!(
		r#"
		SELECT event_id, status::text AS "status!"
		FROM virustotal_file_scans
		WHERE file_id = $1
		"#,
		file_id.0,
	)
	.fetch_optional(pool)
	.await
	.wrap_err("checking existing VirusTotal scan")?;

	if !existing
		.is_some_and(|row| row.event_id == scan_id && row.status == "queued")
	{
		return commit_message(consumer, message);
	}

	let storage_key = event_storage_key(&file.url)?;
	let file_data = match file_host
		.read_file(&storage_key, FileHostPublicity::Public)
		.await
	{
		Ok(data) => data,
		Err(error) => {
			record_submission_failure(
				pool,
				file_id,
				file.project_id,
				file.version_id,
				scan_id,
				&format!("{error:#}"),
				redis_pool,
			)
			.await?;
			return commit_message(consumer, message);
		}
	};
	let sha256 = hex::encode(Sha256::digest(&file_data));
	let updated = sqlx::query!(
		"UPDATE virustotal_file_scans SET sha256 = $2 WHERE file_id = $1 AND event_id = $3 AND status = 'queued'",
		file_id.0,
		sha256,
		scan_id,
	)
	.execute(pool)
	.await
	.wrap_err("recording uploaded file hash for VirusTotal")?;
	if updated.rows_affected() == 0 {
		return commit_message(consumer, message);
	}

	wait_for_active_analysis_slot(pool, client, redis_pool).await?;
	let mut last_error = String::new();
	let mut analysis_id = None;
	for attempt in 1..=MAX_SUBMISSION_ATTEMPTS {
		let updated = sqlx::query!(
			"UPDATE virustotal_file_scans SET submission_attempts = $2, updated_at = NOW() WHERE file_id = $1 AND event_id = $3",
			file_id.0,
			attempt,
			scan_id,
		)
		.execute(pool)
		.await
		.wrap_err("recording VirusTotal submission attempt")?;
		if updated.rows_affected() == 0 {
			return commit_message(consumer, message);
		}

		let upload_result = if file_data.len() > MAX_DIRECT_UPLOAD_BYTES {
			reserve_api_request(pool).await?;
			match client.get_upload_url().await {
				Ok(url) => {
					reserve_api_request(pool).await?;
					client
						.upload_file(
							Some(&url),
							&file.filename,
							file_data.clone(),
						)
						.await
				}
				Err(err) => Err(err),
			}
		} else {
			reserve_api_request(pool).await?;
			client
				.upload_file(None, &file.filename, file_data.clone())
				.await
		};

		match upload_result {
			Ok(id) => {
				analysis_id = Some(id);
				break;
			}
			Err(err) => {
				last_error = format!("{err:#}");
				if attempt < MAX_SUBMISSION_ATTEMPTS {
					let delay = if last_error.contains("rate limit") {
						Duration::from_secs(60)
					} else {
						Duration::from_secs(2u64.pow(attempt as u32))
					};
					tokio::time::sleep(delay).await;
				}
			}
		}
	}

	if let Some(analysis_id) = analysis_id {
		let poll_after = ENV.VIRUSTOTAL_POLL_INTERVAL_SECONDS.max(1) as i64;
		sqlx::query!(
			r#"
			UPDATE virustotal_file_scans
			SET analysis_id = $2,
				status = 'submitted',
				submitted_at = NOW(),
				next_poll_at = NOW() + make_interval(secs => $3),
				error = NULL,
				updated_at = NOW()
			WHERE file_id = $1 AND event_id = $4
			"#,
			file_id.0,
			analysis_id,
			poll_after as f64,
			scan_id,
		)
		.execute(pool)
		.await
		.wrap_err("saving VirusTotal analysis ID")?;
	} else {
		record_submission_failure(
			pool,
			file_id,
			file.project_id,
			file.version_id,
			scan_id,
			&last_error,
			redis_pool,
		)
		.await?;
	}

	commit_message(consumer, message)
}

async fn record_submission_failure(
	pool: &PgPool,
	file_id: DBFileId,
	project_id: crate::database::models::DBProjectId,
	version_id: DBVersionId,
	scan_id: Uuid,
	error: &str,
	redis_pool: &RedisPool,
) -> Result<()> {
	let mut transaction = pool
		.begin()
		.await
		.wrap_err("beginning failed scan transaction")?;
	let before = tech_review_queue::version_malware_statuses(
		&mut transaction,
		&[version_id],
	)
	.await?;
	let updated = sqlx::query!(
		"UPDATE virustotal_file_scans SET status = 'failed', error = $2, updated_at = NOW() WHERE file_id = $1 AND event_id = $3",
		file_id.0,
		error,
		scan_id,
	)
	.execute(&mut transaction)
	.await
	.wrap_err("recording failed VirusTotal submission")?;
	if updated.rows_affected() != 0 {
		tech_review_queue::sync_projects(
			&[project_id],
			tech_review_queue::TechReviewRemovalReason::ScanCompleted,
			&mut transaction,
		)
		.await?;
		let after = tech_review_queue::version_malware_statuses(
			&mut transaction,
			&[version_id],
		)
		.await?;
		tech_review_queue::notify_version_malware_status_changes(
			&before,
			&after,
			&mut transaction,
			redis_pool,
		)
		.await?;
	}
	transaction
		.commit()
		.await
		.wrap_err("committing failed scan")?;
	Ok(())
}

async fn wait_for_file_transaction(
	pool: &PgPool,
	file_id: DBFileId,
) -> Result<bool> {
	let mut transaction = pool
		.begin()
		.await
		.wrap_err("beginning file existence transaction")?;
	AdvisoryLock::VirusTotalFile(file_id)
		.acquire(&mut transaction)
		.await
		.wrap_err("waiting for version upload transaction")?;
	let exists = sqlx::query_scalar!(
		"SELECT EXISTS(SELECT 1 FROM files WHERE id = $1) AS \"exists!\"",
		file_id.0,
	)
	.fetch_one(&mut transaction)
	.await
	.wrap_err("checking VirusTotal scan file existence")?;
	transaction
		.commit()
		.await
		.wrap_err("committing file existence transaction")?;
	Ok(exists)
}

fn event_storage_key(file_url: &str) -> Result<String> {
	let key = file_url
		.strip_prefix(&ENV.CDN_URL)
		.unwrap_or(file_url)
		.trim_start_matches('/');
	let decoded =
		urlencoding::decode(key).wrap_err("decoding file storage key")?;
	Ok(decoded.into_owned())
}

async fn wait_for_active_analysis_slot(
	pool: &PgPool,
	client: &VirusTotalClient,
	redis_pool: &RedisPool,
) -> Result<()> {
	loop {
		let active_count = sqlx::query_scalar!(
			"SELECT COUNT(*) FROM virustotal_file_scans WHERE status = 'submitted'",
		)
		.fetch_one(pool)
		.await
		.wrap_err("counting active VirusTotal analyses")?
		.unwrap_or_default();
		if active_count < ENV.VIRUSTOTAL_MAX_ACTIVE_ANALYSES.max(1) {
			return Ok(());
		}
		poll_submitted_analyses(pool, client, redis_pool).await?;
		tokio::time::sleep(Duration::from_secs(
			ENV.VIRUSTOTAL_POLL_INTERVAL_SECONDS.max(1),
		))
		.await;
	}
}

async fn reserve_api_request(pool: &PgPool) -> Result<()> {
	loop {
		let mut transaction = pool
			.begin()
			.await
			.wrap_err("beginning VirusTotal quota transaction")?;
		AdvisoryLock::VirusTotalApiQuota
			.acquire(&mut transaction)
			.await
			.wrap_err("locking shared VirusTotal API quota")?;
		let usage = sqlx::query!(
			"SELECT minute_started_at, minute_requests, day_started_at, day_requests FROM virustotal_api_usage WHERE id = TRUE FOR UPDATE",
		)
		.fetch_one(&mut transaction)
		.await
		.wrap_err("reading VirusTotal API quota")?;
		let now = Utc::now();
		let minute_start = if now
			.signed_duration_since(usage.minute_started_at)
			.num_seconds()
			>= 60
		{
			now
		} else {
			usage.minute_started_at
		};
		let day_start = if now.date_naive() != usage.day_started_at.date_naive()
		{
			now
		} else {
			usage.day_started_at
		};
		let minute_count = if minute_start == now {
			0
		} else {
			usage.minute_requests
		};
		let day_count = if day_start == now {
			0
		} else {
			usage.day_requests
		};
		let mut wait = Duration::ZERO;
		if minute_count >= ENV.VIRUSTOTAL_REQUESTS_PER_MINUTE.max(1) as i32 {
			wait = wait.max(
				(minute_start + chrono::Duration::minutes(1) - now)
					.to_std()
					.unwrap_or_default(),
			);
		}
		if day_count >= ENV.VIRUSTOTAL_REQUESTS_PER_DAY.max(1) as i32 {
			wait = wait.max(
				(day_start + chrono::Duration::days(1) - now)
					.to_std()
					.unwrap_or_default(),
			);
		}
		if wait.is_zero() {
			sqlx::query!(
				"UPDATE virustotal_api_usage SET minute_started_at = $1, minute_requests = $2, day_started_at = $3, day_requests = $4 WHERE id = TRUE",
				minute_start,
				minute_count + 1,
				day_start,
				day_count + 1,
			)
			.execute(&mut transaction)
			.await
			.wrap_err("recording VirusTotal API quota usage")?;
			transaction
				.commit()
				.await
				.wrap_err("committing VirusTotal API quota")?;
			return Ok(());
		}
		transaction
			.commit()
			.await
			.wrap_err("committing VirusTotal API quota wait")?;
		tokio::time::sleep(wait).await;
	}
}

async fn poll_submitted_analyses(
	pool: &PgPool,
	client: &VirusTotalClient,
	redis_pool: &RedisPool,
) -> Result<()> {
	let rows = sqlx::query!(
		r#"
		SELECT scan.file_id AS "file_id!: DBFileId",
			scan.analysis_id AS "analysis_id!", scan.event_id, scan.poll_attempts,
			version.mod_id AS "project_id!: crate::database::models::DBProjectId",
			version.id AS "version_id!: DBVersionId"
		FROM virustotal_file_scans scan
		INNER JOIN files ON files.id = scan.file_id
		INNER JOIN versions version ON version.id = files.version_id
		WHERE scan.status = 'submitted' AND scan.next_poll_at <= NOW()
		ORDER BY scan.next_poll_at, scan.file_id
		LIMIT $1
		"#,
		ENV.VIRUSTOTAL_MAX_ACTIVE_ANALYSES.max(1),
	)
	.fetch_all(pool)
	.await
	.wrap_err("fetching VirusTotal analyses due for polling")?;

	for row in rows {
		reserve_api_request(pool).await?;
		match client.get_analysis(&row.analysis_id).await {
			Ok(analysis) => {
				let api_status =
					analysis["status"].as_str().unwrap_or("unknown");
				let scan_status = match api_status {
					"queued" | "in-progress" => "submitted",
					"completed" => "completed",
					_ => "unknown",
				};
				let poll_after =
					ENV.VIRUSTOTAL_POLL_INTERVAL_SECONDS.max(1) as i64;
				let mut transaction = pool
					.begin()
					.await
					.wrap_err("beginning scan result transaction")?;
				let before = tech_review_queue::version_malware_statuses(
					&mut transaction,
					&[row.version_id],
				)
				.await?;
				let updated = sqlx::query!(
					r#"
					UPDATE virustotal_file_scans
					SET status = CASE
							WHEN $2 THEN 'submitted'::virustotal_scan_status
							WHEN $3 THEN 'completed'::virustotal_scan_status
							ELSE 'unknown'::virustotal_scan_status
						END,
						analysis = $4,
						checked_at = NOW(),
						next_poll_at = CASE WHEN $2 THEN NOW() + make_interval(secs => $5) ELSE NULL END,
						error = CASE WHEN NOT $2 AND NOT $3 THEN 'VirusTotal returned an unknown analysis status' ELSE NULL END,
						poll_attempts = 0,
						updated_at = NOW()
					WHERE file_id = $1 AND event_id = $6 AND status = 'submitted'
					"#,
					row.file_id.0,
					scan_status == "submitted",
					scan_status == "completed",
					analysis.clone(),
					poll_after as f64,
					row.event_id,
				)
				.execute(&mut transaction)
				.await
				.wrap_err("saving VirusTotal analysis result")?;

				if scan_status != "submitted" && updated.rows_affected() != 0 {
					tech_review_queue::sync_projects(
						&[row.project_id],
						tech_review_queue::TechReviewRemovalReason::ScanCompleted,
						&mut transaction,
					)
					.await?;
					let after = tech_review_queue::version_malware_statuses(
						&mut transaction,
						&[row.version_id],
					)
					.await?;
					tech_review_queue::notify_version_malware_status_changes(
						&before,
						&after,
						&mut transaction,
						redis_pool,
					)
					.await?;
				}
				transaction
					.commit()
					.await
					.wrap_err("committing VirusTotal analysis result")?;
			}
			Err(err) => {
				let attempts = row.poll_attempts + 1;
				let should_fail = attempts >= MAX_POLL_ATTEMPTS;
				let next_poll =
					ENV.VIRUSTOTAL_POLL_INTERVAL_SECONDS.max(1) as i64;
				let error = format!("{err:#}");
				let mut transaction = pool
					.begin()
					.await
					.wrap_err("beginning scan polling error transaction")?;
				let before = tech_review_queue::version_malware_statuses(
					&mut transaction,
					&[row.version_id],
				)
				.await?;
				let updated = sqlx::query!(
					r#"
					UPDATE virustotal_file_scans
					SET status = CASE WHEN $2 THEN 'failed'::virustotal_scan_status ELSE status END,
						error = $3,
						poll_attempts = $4,
						checked_at = NOW(),
						next_poll_at = CASE WHEN $2 THEN NULL ELSE NOW() + make_interval(secs => $5) END,
						updated_at = NOW()
					WHERE file_id = $1 AND event_id = $6 AND status = 'submitted'
					"#,
					row.file_id.0,
					should_fail,
					&error,
					attempts,
					next_poll as f64,
					row.event_id,
				)
				.execute(&mut transaction)
				.await
				.wrap_err("recording VirusTotal polling error")?;
				if should_fail && updated.rows_affected() != 0 {
					tech_review_queue::sync_projects(
						&[row.project_id],
						tech_review_queue::TechReviewRemovalReason::ScanCompleted,
						&mut transaction,
					).await?;
					let after = tech_review_queue::version_malware_statuses(
						&mut transaction,
						&[row.version_id],
					)
					.await?;
					tech_review_queue::notify_version_malware_status_changes(
						&before,
						&after,
						&mut transaction,
						redis_pool,
					)
					.await?;
				}
				transaction
					.commit()
					.await
					.wrap_err("committing VirusTotal polling error")?;
				warn!(
					file_id = row.file_id.0,
					attempt = attempts,
					"VirusTotal analysis poll failed: {err:#}"
				);
			}
		}
	}
	Ok(())
}

fn commit_message(
	consumer: &StreamConsumer,
	message: &BorrowedMessage<'_>,
) -> Result<()> {
	consumer
		.commit_message(message, CommitMode::Sync)
		.wrap_err("committing VirusTotal file scan Kafka message")
}
