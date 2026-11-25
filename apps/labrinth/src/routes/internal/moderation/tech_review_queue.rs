use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde_json::Value;

use crate::{
	database::{
		Executor, PgPool, PgTransaction,
		advisory_lock::AdvisoryLock,
		models::{
			DBFileId, DBProjectId, DBThreadId, DBVersionId,
			thread_item::ThreadMessageBuilder,
		},
	},
	env::ENV,
	models::{
		projects::{FileMalwareScan, MalwareStatus, Version, VersionStatus},
		threads::MessageBody,
	},
	routes::ApiError,
	util::error::Context,
};

#[derive(Debug, Clone, Copy)]
pub enum TechReviewRemovalReason {
	ScanCompleted,
	FileDeleted,
	ReviewCompleted,
}

pub async fn add_projects(
	project_ids: &[DBProjectId],
	txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
	let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
	if project_ids.is_empty() {
		return Ok(());
	}

	let rows = sqlx::query!(
		r#"
		WITH inserted AS (
			INSERT INTO technical_review_queue (project_id)
			SELECT unnest($1::bigint[])
			ON CONFLICT (project_id) DO NOTHING
			RETURNING project_id
		)
		SELECT inserted.project_id AS "project_id!: DBProjectId",
			(
				SELECT thread.id
				FROM threads thread
				WHERE thread.mod_id = inserted.project_id
				ORDER BY thread.id
				LIMIT 1
			) AS "thread_id!: DBThreadId"
		FROM inserted
		"#,
		&project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
	)
	.fetch_all(&mut *txn)
	.await
	.wrap_internal_err("adding projects to technical review queue")?;

	for row in rows {
		ThreadMessageBuilder {
			author_id: None,
			body: MessageBody::TechReviewEntered,
			thread_id: row.thread_id,
			hide_identity: false,
		}
		.insert(txn)
		.await
		.wrap_internal_err("adding technical review queue thread message")?;
	}
	Ok(())
}

pub async fn remove_projects(
	project_ids: &[DBProjectId],
	reason: TechReviewRemovalReason,
	txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
	let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
	if project_ids.is_empty() {
		return Ok(());
	}

	let rows = sqlx::query!(
		r#"
		WITH removed AS (
			DELETE FROM technical_review_queue
			WHERE project_id = ANY($1::bigint[])
			RETURNING project_id
		)
		SELECT removed.project_id AS "project_id!: DBProjectId",
			(
				SELECT thread.id
				FROM threads thread
				WHERE thread.mod_id = removed.project_id
				ORDER BY thread.id
				LIMIT 1
			) AS "thread_id!: DBThreadId"
		FROM removed
		"#,
		&project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
	)
	.fetch_all(&mut *txn)
	.await
	.wrap_internal_err("removing projects from technical review queue")?;

	let body = match reason {
		TechReviewRemovalReason::ScanCompleted
		| TechReviewRemovalReason::ReviewCompleted => MessageBody::TechReviewExited,
		TechReviewRemovalReason::FileDeleted => {
			MessageBody::TechReviewExitFileDeleted
		}
	};
	for row in rows {
		ThreadMessageBuilder {
			author_id: None,
			body: body.clone(),
			thread_id: row.thread_id,
			hide_identity: false,
		}
		.insert(txn)
		.await
		.wrap_internal_err("adding technical review exit thread message")?;
	}
	Ok(())
}

pub(crate) struct FileScanResult {
	pub project_id: DBProjectId,
	version_id: DBVersionId,
	pub version_status: VersionStatus,
	pub malware: FileMalwareScan,
	pub unresolved: bool,
	pub confirmed: bool,
}

pub(crate) async fn file_scan_results<'a>(
	executor: impl Executor<'a, Database = sqlx::Postgres>,
	version_ids: &[DBVersionId],
	project_ids: &[DBProjectId],
) -> Result<HashMap<DBFileId, FileScanResult>, ApiError> {
	if version_ids.is_empty() && project_ids.is_empty() {
		return Ok(HashMap::new());
	}

	let rows = sqlx::query!(
		r#"
		SELECT f.id, v.mod_id, f.version_id, v.status AS version_status,
			s.status::text AS "scan_status?", s.analysis_id AS "analysis_id?",
			s.submitted_at AS "submitted_at?",
			s.analysis AS "analysis?: sqlx::types::Json<Value>",
			(
				SELECT jsonb_agg(jsonb_build_array(d.engine_name, d.category, d.result, d.verdict::text))
				FROM virustotal_finding_verdicts d
				WHERE d.scope = 'local' AND d.file_id = f.id AND d.sha256 = s.sha256
					AND d.updated_at >= s.checked_at
			) AS "decisions?: sqlx::types::Json<Vec<[String; 4]>>"
		FROM versions v
		INNER JOIN files f ON f.version_id = v.id
		LEFT JOIN virustotal_file_scans s ON s.file_id = f.id
		WHERE v.id = ANY($1::bigint[]) OR v.mod_id = ANY($2::bigint[])
		"#,
		&version_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
		&project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
	)
	.fetch_all(executor)
	.await
	.wrap_internal_err("reading file malware scans")?;

	Ok(rows
		.into_iter()
		.map(|row| {
			let mut result = FileScanResult {
				project_id: DBProjectId(row.mod_id),
				version_id: DBVersionId(row.version_id),
				version_status: VersionStatus::from_string(&row.version_status),
				malware: FileMalwareScan {
					status: MalwareStatus::Pending,
					analysis_url: row
						.analysis_id
						.as_ref()
						.filter(|_| row.submitted_at.is_some())
						.map(|id| {
							format!(
								"https://www.virustotal.com/gui/file-analysis/{id}"
							)
						}),
					failed: matches!(
						row.scan_status.as_deref(),
						Some("failed" | "unknown")
					),
				},
				unresolved: false,
				confirmed: false,
			};
			if row.scan_status.as_deref() == Some("completed")
				&& result.malware.analysis_url.is_some()
			{
				let decisions = row
					.decisions
					.map(|decisions| decisions.0)
					.unwrap_or_default();
				match row.analysis.as_ref().and_then(|analysis| {
					assess_analysis(&analysis.0, &decisions)
				}) {
					Some((status, unresolved, confirmed)) => {
						result.malware.status = status;
						result.unresolved = unresolved;
						result.confirmed = confirmed;
					}
					None => result.malware.failed = true,
				}
			}
			(DBFileId(row.id), result)
		})
		.collect())
}

fn assess_analysis(
	analysis: &Value,
	decisions: &[[String; 4]],
) -> Option<(MalwareStatus, bool, bool)> {
	let malicious = analysis["stats"]["malicious"].as_u64()?;
	let suspicious = analysis["stats"]["suspicious"].as_u64()?;
	let findings = analysis["results"].as_object()?;
	if findings.is_empty() {
		return None;
	}
	let threshold = u64::from(ENV.VIRUSTOTAL_DETECTION_THRESHOLD.max(1));
	let mut unresolved = false;
	let mut confirmed = false;
	let mut manually_approved = false;
	let mut clean_result = false;
	let mut malicious_seen = 0;
	let mut suspicious_seen = 0;
	for (key, finding) in findings {
		let category = finding["category"].as_str()?;
		match category {
			"malicious" => malicious_seen += 1,
			"suspicious" => suspicious_seen += 1,
			"harmless" | "undetected" => {
				clean_result = true;
				continue;
			}
			_ => continue,
		}
		if malicious < threshold && suspicious == 0 {
			continue;
		}
		let engine = finding["engine_name"].as_str().unwrap_or(key);
		let result = finding["result"].as_str().unwrap_or_default();
		match decisions
			.iter()
			.find(|decision| {
				decision[0] == engine
					&& decision[1] == category
					&& decision[2] == result
			})
			.map(|decision| decision[3].as_str())
		{
			Some("safe") => manually_approved = true,
			Some("unsafe") => confirmed = true,
			_ => unresolved = true,
		}
	}
	if malicious_seen != malicious
		|| suspicious_seen != suspicious
		|| (!clean_result && malicious_seen == 0 && suspicious_seen == 0)
	{
		return None;
	}
	let status = if unresolved || confirmed {
		MalwareStatus::Blocked
	} else if manually_approved {
		MalwareStatus::ManuallyApproved
	} else {
		MalwareStatus::Passed
	};
	Some((status, unresolved, confirmed))
}

pub(crate) async fn attach_file_scans(
	pool: &PgPool,
	versions: &mut [Version],
) -> Result<(), ApiError> {
	let version_ids = versions
		.iter()
		.filter(|version| !version.files.is_empty())
		.map(|version| version.id.into())
		.collect::<Vec<_>>();
	let mut scans = file_scan_results(pool, &version_ids, &[]).await?;
	let counts = scans.values().counts_by(|scan| scan.version_id);
	for version in versions {
		let version_id = version.id.into();
		if counts.get(&version_id).copied().unwrap_or_default()
			== version.files.len()
		{
			for file in &mut version.files {
				if let Some(scan) =
					file.id.and_then(|id| scans.remove(&id.into()))
				{
					file.malware = scan.malware;
				}
			}
		}
	}
	Ok(())
}

pub async fn sync_projects(
	project_ids: &[DBProjectId],
	removal_reason: TechReviewRemovalReason,
	txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
	let project_ids = project_ids
		.iter()
		.copied()
		.unique()
		.sorted_unstable_by_key(|id| id.0)
		.collect::<Vec<_>>();
	lock_projects(&project_ids, txn).await?;
	let scans = file_scan_results(&mut *txn, &[], &project_ids).await?;
	let needs_review = scans
		.values()
		.filter(|scan| {
			scan.malware.status == MalwareStatus::Blocked || scan.malware.failed
		})
		.map(|scan| scan.project_id)
		.collect::<HashSet<_>>();
	let (waiting, finished): (Vec<_>, Vec<_>) = project_ids
		.into_iter()
		.partition(|project_id| needs_review.contains(project_id));
	add_projects(&waiting, txn).await?;
	remove_projects(&finished, removal_reason, txn).await
}

pub async fn reconcile_cleared_projects(pool: &PgPool) -> Result<(), ApiError> {
	let project_ids = sqlx::query_scalar!(
		r#"SELECT project_id AS "project_id!: DBProjectId" FROM technical_review_queue ORDER BY project_id"#,
	)
	.fetch_all(pool)
	.await
	.wrap_internal_err("reading existing technical review queue entries")?;

	for project_ids in project_ids.chunks(32) {
		let mut txn = pool.begin().await.wrap_internal_err(
			"starting technical review queue reconciliation",
		)?;
		lock_projects(project_ids, &mut txn).await?;
		let scans = file_scan_results(&mut txn, &[], project_ids).await?;
		let awaiting_clearance = scans
			.values()
			.filter(|scan| !scan.malware.status.allows_download())
			.map(|scan| scan.project_id)
			.collect::<HashSet<_>>();
		let cleared = project_ids
			.iter()
			.copied()
			.filter(|id| !awaiting_clearance.contains(id))
			.collect::<Vec<_>>();
		remove_projects(
			&cleared,
			TechReviewRemovalReason::ScanCompleted,
			&mut txn,
		)
		.await?;
		txn.commit().await.wrap_internal_err(
			"committing technical review queue reconciliation",
		)?;
	}
	Ok(())
}

async fn lock_projects(
	project_ids: &[DBProjectId],
	txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
	for &project_id in project_ids {
		AdvisoryLock::VirusTotalProject(project_id)
			.acquire(txn)
			.await
			.wrap_internal_err(
				"locking project for technical review queue sync",
			)?;
	}
	Ok(())
}
