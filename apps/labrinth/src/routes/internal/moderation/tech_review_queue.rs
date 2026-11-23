use itertools::Itertools;

use crate::{
	database::{
		PgTransaction,
		models::{DBProjectId, DBThreadId, thread_item::ThreadMessageBuilder},
	},
	env::ENV,
	models::threads::MessageBody,
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

pub async fn sync_projects(
	project_ids: &[DBProjectId],
	removal_reason: TechReviewRemovalReason,
	txn: &mut PgTransaction<'_>,
) -> Result<(), ApiError> {
	let project_ids = project_ids.iter().copied().unique().collect::<Vec<_>>();
	if project_ids.is_empty() {
		return Ok(());
	}

	let rows = sqlx::query!(
		r#"
		WITH findings AS (
			SELECT
				version.mod_id AS project_id,
				scan.file_id,
				scan.sha256,
				COALESCE(result.value->>'engine_name', result.key) AS engine_name,
				COALESCE(result.value->>'category', '') AS category,
				COALESCE(result.value->>'result', '') AS result
			FROM virustotal_file_scans scan
			INNER JOIN files ON files.id = scan.file_id
			INNER JOIN versions version ON version.id = files.version_id
			CROSS JOIN LATERAL jsonb_each(COALESCE(scan.analysis->'results', '{}'::jsonb)) result
			WHERE scan.status = 'completed'
				AND (
					COALESCE((scan.analysis->'stats'->>'malicious')::integer, 0) >= $2
					OR COALESCE((scan.analysis->'stats'->>'suspicious')::integer, 0) > 0
				)
				AND result.value->>'category' IN ('malicious', 'suspicious')
		), decisions AS (
			SELECT
				finding.project_id,
				finding.file_id,
				finding.sha256,
				COALESCE(local_verdict.verdict, global_verdict.verdict)::text AS verdict
			FROM findings finding
			LEFT JOIN virustotal_finding_verdicts local_verdict
				ON local_verdict.scope = 'local'
				AND local_verdict.file_id = finding.file_id
				AND local_verdict.engine_name = finding.engine_name
				AND local_verdict.category = finding.category
				AND local_verdict.result = finding.result
			LEFT JOIN virustotal_finding_verdicts global_verdict
				ON global_verdict.scope = 'global'
				AND global_verdict.sha256 = finding.sha256
				AND global_verdict.engine_name = finding.engine_name
				AND global_verdict.category = finding.category
				AND global_verdict.result = finding.result
		)
		SELECT
			requested.project_id AS "project_id!: DBProjectId",
			COALESCE(BOOL_OR(decisions.verdict IS NULL OR decisions.verdict = 'unsafe'), FALSE) AS "needs_review!"
		FROM unnest($1::bigint[]) requested(project_id)
		LEFT JOIN decisions ON decisions.project_id = requested.project_id
		GROUP BY requested.project_id
		"#,
		&project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
		ENV.VIRUSTOTAL_DETECTION_THRESHOLD as i32,
	)
	.fetch_all(&mut *txn)
	.await
	.wrap_internal_err("checking technical review queue membership")?;

	add_projects(
		&rows
			.iter()
			.filter(|row| row.needs_review)
			.map(|row| row.project_id)
			.collect::<Vec<_>>(),
		txn,
	)
	.await?;
	remove_projects(
		&rows
			.into_iter()
			.filter(|row| !row.needs_review)
			.map(|row| row.project_id)
			.collect::<Vec<_>>(),
		removal_reason,
		txn,
	)
	.await
}
