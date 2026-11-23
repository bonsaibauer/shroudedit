use std::{collections::HashMap, fmt};

use actix_web::{HttpRequest, get, patch, post, web};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use xredis::RedisPool;

use super::{Ownership, ownership::get_projects_ownership, tech_review_queue};
use crate::{
	auth::check_is_moderator_from_headers,
	database::{
		PgPool,
		models::{
			DBFileId, DBProject, DBProjectId, DBThread, DBThreadId, DBUser,
			DBUserId, thread_item::ThreadMessageBuilder,
		},
	},
	env::ENV,
	models::{
		ids::{FileId, ProjectId, ThreadId, VersionId},
		pats::Scopes,
		projects::{Project, ProjectStatus},
		threads::{MessageBody, TechReviewVerdict, Thread},
	},
	queue::session::AuthQueue,
	routes::ApiError,
	search::SearchState,
	util::error::{ApiContext as _, Context as _},
};

pub fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(search_projects)
		.service(get_project_report)
		.service(submit_report)
		.service(update_finding_verdicts);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchProjects {
	#[serde(default = "default_limit")]
	pub limit: u64,
	#[serde(default)]
	pub page: u64,
	#[serde(default)]
	pub filter: SearchProjectsFilter,
	#[serde(default = "default_sort_by")]
	pub sort_by: SearchProjectsSort,
}

fn default_limit() -> u64 {
	50
}

fn default_sort_by() -> SearchProjectsSort {
	SearchProjectsSort::CreatedAsc
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchProjectsFilter {
	#[serde(default)]
	pub project_type: Vec<String>,
	#[serde(default)]
	pub replied_to: Option<RepliedTo>,
	#[serde(default)]
	pub project_status: Vec<ProjectStatus>,
}

#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Deserialize,
	utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RepliedTo {
	Replied,
	Unreplied,
}

#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	Hash,
	Serialize,
	Deserialize,
	utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SearchProjectsSort {
	CreatedAsc,
	CreatedDesc,
	DetectionCountAsc,
	DetectionCountDesc,
}

impl fmt::Display for SearchProjectsSort {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		let value = serde_json::to_value(*self).unwrap();
		write!(formatter, "{}", value.as_str().unwrap())
	}
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VirusTotalScanStatus {
	Queued,
	Submitted,
	Completed,
	Failed,
	Unknown,
}

impl VirusTotalScanStatus {
	fn from_db(status: &str) -> Self {
		match status {
			"submitted" => Self::Submitted,
			"completed" => Self::Completed,
			"failed" => Self::Failed,
			"unknown" => Self::Unknown,
			_ => Self::Queued,
		}
	}
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum VirusTotalVerdict {
	Safe,
	Unsafe,
}

impl VirusTotalVerdict {
	fn as_str(self) -> &'static str {
		match self {
			Self::Safe => "safe",
			Self::Unsafe => "unsafe",
		}
	}

	fn from_db(verdict: &str) -> Option<Self> {
		match verdict {
			"safe" => Some(Self::Safe),
			"unsafe" => Some(Self::Unsafe),
			_ => None,
		}
	}
}

#[derive(
	Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FindingVerdictScope {
	Local,
	Global,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VirusTotalFinding {
	pub engine_name: String,
	pub category: String,
	pub result: String,
	pub local_verdict: Option<VirusTotalVerdict>,
	pub global_verdict: Option<VirusTotalVerdict>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FileReport {
	pub file_id: FileId,
	pub file_name: String,
	pub file_size: i32,
	pub download_url: String,
	pub sha256: Option<String>,
	pub status: VirusTotalScanStatus,
	pub analysis_id: Option<String>,
	pub analysis_url: Option<String>,
	pub checked_at: Option<DateTime<Utc>>,
	pub error: Option<String>,
	pub analysis: Option<Value>,
	pub findings: Vec<VirusTotalFinding>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReport {
	pub project_id: ProjectId,
	pub max_detection_count: Option<u64>,
	pub versions: Vec<VersionReport>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct VersionReport {
	pub version_id: VersionId,
	pub version_number: String,
	pub files: Vec<FileReport>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectReportResponse {
	pub project_report: Option<ProjectReport>,
	pub thread: Thread,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ProjectModerationInfo {
	pub id: ProjectId,
	pub thread_id: ThreadId,
	pub name: String,
	pub status: ProjectStatus,
	#[serde(default)]
	pub project_types: Vec<String>,
	pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SearchResponse {
	pub project_reports: Vec<ProjectReport>,
	pub projects: HashMap<ProjectId, ProjectModerationInfo>,
	pub threads: HashMap<ThreadId, Thread>,
	pub ownership: HashMap<ProjectId, Ownership>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateFindingVerdict {
	pub file_id: FileId,
	pub sha256: String,
	pub engine_name: String,
	pub category: String,
	pub result: String,
	pub scope: FindingVerdictScope,
	pub verdict: Option<VirusTotalVerdict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SubmitReport {
	pub verdict: TechReviewVerdict,
	pub message: Option<String>,
}

#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = SearchProjects,
	responses((status = OK, body = inline(SearchResponse)))
)]
#[post("/search")]
pub async fn search_projects(
	req: HttpRequest,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
	search_req: web::Json<SearchProjects>,
) -> Result<web::Json<SearchResponse>, ApiError> {
	let user = check_is_moderator_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_READ,
	)
	.await
	.wrap_auth_err("authenticating API request")?;

	let search_req = search_req.into_inner();
	let sort_by = search_req.sort_by.to_string();
	let limit = i64::try_from(search_req.limit.clamp(1, 100))
		.wrap_request_err("limit cannot fit into `i64`")?;
	let offset = i64::try_from(limit as u64 * search_req.page)
		.wrap_request_err("offset cannot fit into `i64`")?;
	let replied_to = search_req.filter.replied_to.map(|reply| match reply {
		RepliedTo::Replied => "replied",
		RepliedTo::Unreplied => "unreplied",
	});
	let project_status = search_req
		.filter
		.project_status
		.iter()
		.map(ToString::to_string)
		.collect::<Vec<_>>();

	let rows = sqlx::query!(
		r#"
		SELECT
			project.id AS "project_id!: DBProjectId",
			MIN(thread.id) AS "thread_id!: DBThreadId"
		FROM technical_review_queue queue
		INNER JOIN mods project ON project.id = queue.project_id
		INNER JOIN threads thread ON thread.mod_id = project.id
		LEFT JOIN versions version ON version.mod_id = project.id
		LEFT JOIN files file ON file.version_id = version.id
		LEFT JOIN virustotal_file_scans scan ON scan.file_id = file.id
		LEFT JOIN LATERAL (
			SELECT message.id, message_author.role
			FROM threads_messages message
			LEFT JOIN users message_author ON message_author.id = message.author_id
			WHERE message.thread_id = thread.id
			ORDER BY message.created DESC
			LIMIT 1
		) last_message ON TRUE
		WHERE project.status NOT IN ('draft', 'rejected', 'withheld')
			AND (cardinality($5::text[]) = 0 OR project.status = ANY($5::text[]))
			AND (
				cardinality($3::text[]) = 0
				OR EXISTS (
					SELECT 1
					FROM versions typed_version
					INNER JOIN loaders_versions loader_version ON loader_version.version_id = typed_version.id
					INNER JOIN loaders_project_types loader_type ON loader_type.joining_loader_id = loader_version.loader_id
					INNER JOIN project_types project_type ON project_type.id = loader_type.joining_project_type_id
					WHERE typed_version.mod_id = project.id AND project_type.name = ANY($3::text[])
				)
			)
			AND (
				$4::text IS NULL
				OR ($4 = 'unreplied' AND (last_message.id IS NULL OR last_message.role IS NULL OR last_message.role NOT IN ('moderator', 'admin')))
				OR ($4 = 'replied' AND last_message.role IN ('moderator', 'admin'))
			)
		GROUP BY project.id
		ORDER BY
			CASE WHEN $2 = 'created_asc' THEN MIN(project.queued) END ASC,
			CASE WHEN $2 = 'created_desc' THEN MIN(project.queued) END DESC,
			CASE WHEN $2 = 'detection_count_asc' THEN MAX(COALESCE((scan.analysis->'stats'->>'malicious')::integer, 0) + COALESCE((scan.analysis->'stats'->>'suspicious')::integer, 0)) END ASC,
			CASE WHEN $2 = 'detection_count_desc' THEN MAX(COALESCE((scan.analysis->'stats'->>'malicious')::integer, 0) + COALESCE((scan.analysis->'stats'->>'suspicious')::integer, 0)) END DESC,
			MIN(project.queued) ASC
		LIMIT $1 OFFSET $6
		"#,
		limit,
		&sort_by,
		&search_req.filter.project_type,
		replied_to.as_deref(),
		&project_status,
		offset,
	)
	.fetch_all(&**pool)
	.await
	.wrap_internal_err("fetching VirusTotal technical review projects")?;

	let project_ids = rows.iter().map(|row| row.project_id).collect::<Vec<_>>();
	let thread_ids = rows.iter().map(|row| row.thread_id).collect::<Vec<_>>();
	let project_reports = fetch_project_reports(&project_ids, &pool)
		.await
		.wrap_api_err("fetching project VirusTotal reports")?;
	let projects = DBProject::get_many_ids(&project_ids, &**pool, &redis)
		.await
		.wrap_internal_err("fetching technical review projects")?
		.into_iter()
		.map(|project| {
			(ProjectId::from(project.inner.id), Project::from(project))
		})
		.collect::<HashMap<_, _>>();
	let db_threads = DBThread::get_many(&thread_ids, &**pool)
		.await
		.wrap_internal_err("fetching technical review threads")?;
	let thread_author_ids = db_threads
		.iter()
		.flat_map(|thread| {
			thread
				.messages
				.iter()
				.filter_map(|message| message.author_id)
		})
		.collect::<Vec<_>>();
	let thread_authors =
		DBUser::get_many_ids(&thread_author_ids, &**pool, &redis)
			.await
			.wrap_internal_err("fetching thread authors")?
			.into_iter()
			.map(From::from)
			.collect::<Vec<_>>();
	let threads = db_threads
		.into_iter()
		.map(|thread| {
			let thread = Thread::from(thread, thread_authors.clone(), &user);
			(thread.id, thread)
		})
		.collect::<HashMap<_, _>>();
	let project_list = projects.values().cloned().collect::<Vec<_>>();
	let ownerships = get_projects_ownership(&project_list, &pool, &redis)
		.await
		.wrap_internal_err("fetching project owners")?;
	let ownership = projects
		.keys()
		.copied()
		.zip(ownerships)
		.collect::<HashMap<_, _>>();
	let projects = projects
		.into_iter()
		.map(|(id, project)| {
			(
				id,
				ProjectModerationInfo {
					id,
					thread_id: project.thread_id,
					name: project.name,
					status: project.status,
					project_types: project.project_types,
					icon_url: project.icon_url,
				},
			)
		})
		.collect();

	Ok(web::Json(SearchResponse {
		project_reports,
		projects,
		threads,
		ownership,
	}))
}

#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	responses((status = OK, body = inline(ProjectReportResponse)))
)]
#[get("/project/{id}")]
pub async fn get_project_report(
	req: HttpRequest,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
	path: web::Path<(ProjectId,)>,
) -> Result<web::Json<ProjectReportResponse>, ApiError> {
	let user = check_is_moderator_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_READ,
	)
	.await
	.wrap_auth_err("authenticating API request")?;
	let project_id = DBProjectId::from(path.into_inner().0);
	let row = sqlx::query!(
		"SELECT id AS \"thread_id!: DBThreadId\" FROM threads WHERE mod_id = $1 ORDER BY id LIMIT 1",
		project_id.0,
	)
	.fetch_optional(&**pool)
	.await
	.wrap_internal_err("fetching project moderation thread")?
	.wrap_not_found_err("project not found")?;
	let project_report = fetch_project_reports(&[project_id], &pool)
		.await?
		.into_iter()
		.next();
	let db_threads = DBThread::get_many(&[row.thread_id], &**pool)
		.await
		.wrap_internal_err("fetching project moderation thread")?;
	let author_ids = db_threads
		.iter()
		.flat_map(|thread| {
			thread
				.messages
				.iter()
				.filter_map(|message| message.author_id)
		})
		.collect::<Vec<_>>();
	let authors = DBUser::get_many_ids(&author_ids, &**pool, &redis)
		.await
		.wrap_internal_err("fetching project moderation thread authors")?
		.into_iter()
		.map(From::from)
		.collect::<Vec<_>>();
	let thread = db_threads
		.into_iter()
		.next()
		.map(|thread| Thread::from(thread, authors, &user))
		.wrap_not_found_err("project moderation thread not found")?;
	Ok(web::Json(ProjectReportResponse {
		project_report,
		thread,
	}))
}

async fn fetch_project_reports(
	project_ids: &[DBProjectId],
	pool: &PgPool,
) -> Result<Vec<ProjectReport>, ApiError> {
	if project_ids.is_empty() {
		return Ok(Vec::new());
	}
	let version_rows = sqlx::query!(
		"SELECT id AS \"id!: crate::database::models::DBVersionId\", version_number, mod_id AS \"project_id!: DBProjectId\" FROM versions WHERE mod_id = ANY($1::bigint[]) ORDER BY id",
		&project_ids.iter().map(|id| id.0).collect::<Vec<_>>(),
	)
	.fetch_all(pool)
	.await
	.wrap_internal_err("fetching technical review version IDs")?;
	let version_ids =
		version_rows.iter().map(|row| row.id.0).collect::<Vec<_>>();
	let file_rows = sqlx::query!(
		r#"
		SELECT
			files.id AS "file_id!: DBFileId",
			files.version_id AS "version_id!: crate::database::models::DBVersionId",
			files.filename,
			files.size,
			files.url,
			scan.sha256,
			scan.analysis_id,
			scan.status::text AS "scan_status?",
			scan.analysis AS "analysis: sqlx::types::Json<Value>",
			scan.checked_at,
			scan.error
		FROM files
		LEFT JOIN virustotal_file_scans scan ON scan.file_id = files.id
		WHERE files.version_id = ANY($1::bigint[])
		ORDER BY files.version_id, files.id
		"#,
		&version_ids,
	)
	.fetch_all(pool)
	.await
	.wrap_internal_err("fetching file VirusTotal results")?;
	let file_ids = file_rows
		.iter()
		.map(|row| row.file_id.0)
		.collect::<Vec<_>>();
	let hashes = file_rows
		.iter()
		.filter_map(|row| row.sha256.clone())
		.collect::<Vec<_>>();
	let decision_rows = sqlx::query!(
		r#"
		SELECT file_id, sha256, engine_name, category, result, scope,
			verdict::text AS "verdict!"
		FROM virustotal_finding_verdicts
		WHERE (scope = 'local' AND file_id = ANY($1::bigint[]))
			OR (scope = 'global' AND sha256 = ANY($2::text[]))
		"#,
		&file_ids,
		&hashes,
	)
	.fetch_all(pool)
	.await
	.wrap_internal_err("fetching VirusTotal finding decisions")?;
	let mut local_verdicts = HashMap::new();
	let mut global_verdicts = HashMap::new();
	for row in decision_rows {
		let verdict = VirusTotalVerdict::from_db(&row.verdict);
		if row.scope == "local" {
			if let Some(file_id) = row.file_id {
				local_verdicts.insert(
					(file_id, row.engine_name, row.category, row.result),
					verdict,
				);
			}
		} else {
			global_verdicts.insert(
				(row.sha256, row.engine_name, row.category, row.result),
				verdict,
			);
		}
	}

	let mut reports = project_ids
		.iter()
		.map(|project_id| {
			(
				*project_id,
				ProjectReport {
					project_id: (*project_id).into(),
					max_detection_count: None,
					versions: version_rows
						.iter()
						.filter(|row| row.project_id == *project_id)
						.map(|row| VersionReport {
							version_id: row.id.into(),
							version_number: row.version_number.clone(),
							files: Vec::new(),
						})
						.collect(),
				},
			)
		})
		.collect::<HashMap<_, _>>();
	let version_to_project = version_rows
		.iter()
		.map(|row| (row.id.0, row.project_id))
		.collect::<HashMap<_, _>>();
	let version_indexes =
		reports
			.iter()
			.flat_map(|(project_id, report)| {
				report.versions.iter().enumerate().map(
					move |(index, version)| {
						((project_id.0, version.version_id.0 as i64), index)
					},
				)
			})
			.collect::<HashMap<_, _>>();
	for row in file_rows {
		let Some(project_id) = version_to_project.get(&row.version_id.0) else {
			continue;
		};
		let analysis = row.analysis.map(|analysis| analysis.0);
		let sha256 = row.sha256;
		let findings = analysis
			.as_ref()
			.and_then(|analysis| analysis.get("results"))
			.and_then(Value::as_object)
			.map(|results| {
				results
					.iter()
					.map(|(key, value)| {
						let engine_name = value
							.get("engine_name")
							.and_then(Value::as_str)
							.unwrap_or(key)
							.to_string();
						let category = value
							.get("category")
							.and_then(Value::as_str)
							.unwrap_or("unknown")
							.to_string();
						let result = value
							.get("result")
							.and_then(Value::as_str)
							.unwrap_or_default()
							.to_string();
						let local_verdict = local_verdicts
							.get(&(
								row.file_id.0,
								engine_name.clone(),
								category.clone(),
								result.clone(),
							))
							.copied()
							.flatten();
						let global_verdict = sha256.as_ref().and_then(|sha| {
							global_verdicts
								.get(&(
									sha.clone(),
									engine_name.clone(),
									category.clone(),
									result.clone(),
								))
								.copied()
								.flatten()
						});
						VirusTotalFinding {
							engine_name,
							category,
							result,
							local_verdict,
							global_verdict,
						}
					})
					.collect::<Vec<_>>()
			})
			.unwrap_or_default();
		let scan_status = row.scan_status.as_deref().unwrap_or("queued");
		let analysis_id = row.analysis_id;
		let file_report = FileReport {
			file_id: row.file_id.into(),
			file_name: row.filename,
			file_size: row.size,
			download_url: row.url,
			sha256,
			status: VirusTotalScanStatus::from_db(scan_status),
			analysis_url: analysis_id.as_ref().map(|id| {
				format!("https://www.virustotal.com/gui/file-analysis/{id}")
			}),
			analysis_id,
			checked_at: row.checked_at,
			error: row.error,
			analysis,
			findings,
		};
		let detection_count = detection_count(&file_report);
		let report =
			reports.get_mut(project_id).expect("project report exists");
		report.max_detection_count = Some(
			report
				.max_detection_count
				.unwrap_or_default()
				.max(detection_count),
		);
		if let Some(index) =
			version_indexes.get(&(project_id.0, row.version_id.0))
		{
			report.versions[*index].files.push(file_report);
		}
	}

	Ok(project_ids
		.iter()
		.filter_map(|project_id| reports.remove(project_id))
		.collect())
}

fn detection_count(file: &FileReport) -> u64 {
	file.analysis
		.as_ref()
		.and_then(|analysis| analysis.get("stats"))
		.map(|stats| {
			stats
				.get("malicious")
				.and_then(Value::as_u64)
				.unwrap_or_default()
				+ stats
					.get("suspicious")
					.and_then(Value::as_u64)
					.unwrap_or_default()
		})
		.unwrap_or_default()
}

#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = Vec<UpdateFindingVerdict>,
	responses((status = NO_CONTENT))
)]
#[patch("/finding-verdicts")]
pub async fn update_finding_verdicts(
	req: HttpRequest,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
	updates: web::Json<Vec<UpdateFindingVerdict>>,
) -> Result<(), ApiError> {
	let user = check_is_moderator_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_WRITE,
	)
	.await
	.wrap_auth_err("authenticating API request")?;
	let moderator_id: DBUserId = user.id.into();
	let mut transaction = pool
		.begin()
		.await
		.wrap_internal_err("starting finding decision transaction")?;
	let mut affected_projects = Vec::new();
	for update in updates.into_inner() {
		if update.sha256.len() != 64
			|| !update.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
		{
			return Err(ApiError::Request(eyre::eyre!("invalid SHA-256 hash")));
		}
		if !matches!(update.category.as_str(), "malicious" | "suspicious") {
			return Err(ApiError::Request(eyre::eyre!(
				"only malicious and suspicious findings can be decided"
			)));
		}
		let file_id = DBFileId::from(update.file_id);
		let row = sqlx::query!(
			r#"
			SELECT scan.sha256, scan.analysis AS "analysis: sqlx::types::Json<Value>", version.mod_id AS "project_id!: DBProjectId"
			FROM virustotal_file_scans scan
			INNER JOIN files ON files.id = scan.file_id
			INNER JOIN versions version ON version.id = files.version_id
			WHERE scan.file_id = $1
			"#,
			file_id.0,
		)
		.fetch_optional(&mut transaction)
		.await
		.wrap_internal_err("fetching VirusTotal finding source")?
		.wrap_not_found_err("VirusTotal finding not found")?;
		if row.sha256.as_deref() != Some(update.sha256.as_str()) {
			return Err(ApiError::Request(eyre::eyre!(
				"finding hash no longer matches the file"
			)));
		}
		let present = row
			.analysis
			.as_ref()
			.and_then(|analysis| analysis.0.get("results"))
			.and_then(Value::as_object)
			.is_some_and(|results| {
				results.iter().any(|(key, value)| {
					let engine = value
						.get("engine_name")
						.and_then(Value::as_str)
						.unwrap_or(key);
					let category = value
						.get("category")
						.and_then(Value::as_str)
						.unwrap_or_default();
					let result = value
						.get("result")
						.and_then(Value::as_str)
						.unwrap_or_default();
					engine == update.engine_name
						&& category == update.category
						&& result == update.result
				})
			});
		if !present {
			return Err(ApiError::Request(eyre::eyre!(
				"finding is not present in the saved VirusTotal analysis"
			)));
		}

		match update.scope {
			FindingVerdictScope::Local => {
				if let Some(verdict) = update.verdict {
					sqlx::query!(
						r#"
						INSERT INTO virustotal_finding_verdicts (
							scope, file_id, sha256, engine_name, category, result, verdict, moderator_id
						)
						VALUES ('local', $1, $2, $3, $4, $5, $6::text::virustotal_finding_verdict, $7)
						ON CONFLICT (file_id, engine_name, category, result) WHERE scope = 'local'
						DO UPDATE SET verdict = EXCLUDED.verdict, moderator_id = EXCLUDED.moderator_id, updated_at = NOW()
						"#,
						file_id.0,
						&update.sha256,
						&update.engine_name,
						&update.category,
						&update.result,
						verdict.as_str(),
						moderator_id.0,
					)
					.execute(&mut transaction)
					.await
					.wrap_internal_err("saving local VirusTotal finding decision")?;
				} else {
					sqlx::query!(
						"DELETE FROM virustotal_finding_verdicts WHERE scope = 'local' AND file_id = $1 AND engine_name = $2 AND category = $3 AND result = $4",
						file_id.0, &update.engine_name, &update.category, &update.result,
					)
					.execute(&mut transaction)
					.await
					.wrap_internal_err("resetting local VirusTotal finding decision")?;
				}
			}
			FindingVerdictScope::Global => {
				if let Some(verdict) = update.verdict {
					sqlx::query!(
						r#"
						INSERT INTO virustotal_finding_verdicts (
							scope, file_id, sha256, engine_name, category, result, verdict, moderator_id
						)
						VALUES ('global', NULL, $1, $2, $3, $4, $5::text::virustotal_finding_verdict, $6)
						ON CONFLICT (sha256, engine_name, category, result) WHERE scope = 'global'
						DO UPDATE SET verdict = EXCLUDED.verdict, moderator_id = EXCLUDED.moderator_id, updated_at = NOW()
						"#,
						&update.sha256,
						&update.engine_name,
						&update.category,
						&update.result,
						verdict.as_str(),
						moderator_id.0,
					)
					.execute(&mut transaction)
					.await
					.wrap_internal_err("saving global VirusTotal finding decision")?;
				} else {
					sqlx::query!(
						"DELETE FROM virustotal_finding_verdicts WHERE scope = 'global' AND sha256 = $1 AND engine_name = $2 AND category = $3 AND result = $4",
						&update.sha256, &update.engine_name, &update.category, &update.result,
					)
					.execute(&mut transaction)
					.await
					.wrap_internal_err("resetting global VirusTotal finding decision")?;
				}
			}
		}
		affected_projects.push(row.project_id);
	}
	tech_review_queue::sync_projects(
		&affected_projects
			.iter()
			.copied()
			.unique()
			.collect::<Vec<_>>(),
		tech_review_queue::TechReviewRemovalReason::ScanCompleted,
		&mut transaction,
	)
	.await?;
	transaction
		.commit()
		.await
		.wrap_internal_err("committing finding decisions")?;
	Ok(())
}

#[utoipa::path(
	context_path = "/moderation/tech-review",
	tag = "moderation",
	security(("bearer_auth" = [])),
	request_body = SubmitReport,
	responses((status = NO_CONTENT))
)]
#[post("/submit/{project_id}")]
pub async fn submit_report(
	req: HttpRequest,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<AuthQueue>,
	search_state: web::Data<SearchState>,
	web::Json(submit): web::Json<SubmitReport>,
	path: web::Path<(ProjectId,)>,
) -> Result<(), ApiError> {
	let user = check_is_moderator_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_WRITE,
	)
	.await
	.wrap_auth_err("authenticating API request")?;
	let project_id = DBProjectId::from(path.into_inner().0);
	let mut transaction = pool
		.begin()
		.await
		.wrap_internal_err("starting project verdict transaction")?;
	let pending = sqlx::query_scalar!(
		r#"
		WITH findings AS (
			SELECT version.mod_id AS project_id, scan.file_id, scan.sha256,
				COALESCE(result.value->>'engine_name', result.key) AS engine_name,
				COALESCE(result.value->>'category', '') AS category,
				COALESCE(result.value->>'result', '') AS result
			FROM virustotal_file_scans scan
			INNER JOIN files ON files.id = scan.file_id
			INNER JOIN versions version ON version.id = files.version_id
			CROSS JOIN LATERAL jsonb_each(COALESCE(scan.analysis->'results', '{}'::jsonb)) result
			WHERE scan.status = 'completed'
				AND version.mod_id = $1
				AND result.value->>'category' IN ('malicious', 'suspicious')
				AND (
					COALESCE((scan.analysis->'stats'->>'malicious')::integer, 0) >= $2
					OR COALESCE((scan.analysis->'stats'->>'suspicious')::integer, 0) > 0
				)
		), decisions AS (
			SELECT findings.file_id
			FROM findings
			LEFT JOIN virustotal_finding_verdicts local_verdict
				ON local_verdict.scope = 'local' AND local_verdict.file_id = findings.file_id
				AND local_verdict.engine_name = findings.engine_name AND local_verdict.category = findings.category AND local_verdict.result = findings.result
			LEFT JOIN virustotal_finding_verdicts global_verdict
				ON global_verdict.scope = 'global' AND global_verdict.sha256 = findings.sha256
				AND global_verdict.engine_name = findings.engine_name AND global_verdict.category = findings.category AND global_verdict.result = findings.result
			WHERE local_verdict.verdict IS NULL AND global_verdict.verdict IS NULL
		)
		SELECT COUNT(*) FROM decisions
		"#,
		project_id.0,
		ENV.VIRUSTOTAL_DETECTION_THRESHOLD as i32,
	)
	.fetch_one(&mut transaction)
	.await
	.wrap_internal_err("checking unresolved VirusTotal findings")?
	.unwrap_or_default();
	if pending > 0 {
		return Err(ApiError::Request(eyre::eyre!(
			"project still has unresolved VirusTotal findings"
		)));
	}

	sqlx::query_scalar!(
		"DELETE FROM technical_review_queue WHERE project_id = $1 RETURNING project_id AS \"project_id!: DBProjectId\"",
		project_id.0,
	)
	.fetch_optional(&mut transaction)
	.await
	.wrap_internal_err("removing project from technical review queue")?
	.wrap_not_found_err("project not found in technical review queue")?;
	let thread_id = sqlx::query_scalar!(
		"SELECT id AS \"id!: DBThreadId\" FROM threads WHERE mod_id = $1 ORDER BY id LIMIT 1",
		project_id.0,
	)
	.fetch_one(&mut transaction)
	.await
	.wrap_internal_err("fetching technical review thread")?;
	if let Some(message) =
		submit.message.filter(|message| !message.trim().is_empty())
	{
		ThreadMessageBuilder {
			author_id: Some(user.id.into()),
			body: MessageBody::Text {
				body: message,
				private: true,
				replying_to: None,
				associated_images: Vec::new(),
			},
			thread_id,
			hide_identity: user.role.is_mod(),
		}
		.insert(&mut transaction)
		.await
		.wrap_internal_err("adding technical review moderator message")?;
	}
	ThreadMessageBuilder {
		author_id: Some(user.id.into()),
		body: MessageBody::TechReview {
			verdict: submit.verdict,
		},
		thread_id,
		hide_identity: user.role.is_mod(),
	}
	.insert(&mut transaction)
	.await
	.wrap_internal_err("recording technical review verdict")?;

	if submit.verdict == TechReviewVerdict::Unsafe {
		let status = sqlx::query!(
			r#"
			WITH old AS (SELECT id, status FROM mods WHERE id = $2),
			updated AS (
				UPDATE mods SET status = $1 FROM threads thread
				WHERE mods.id = $2 AND thread.mod_id = mods.id
				RETURNING mods.id, thread.id AS thread_id
			)
			SELECT updated.thread_id AS "thread_id!: DBThreadId", old.status AS "old_status!"
			FROM updated INNER JOIN old ON old.id = updated.id
			"#,
			ProjectStatus::Rejected.as_str(),
			project_id.0,
		)
		.fetch_one(&mut transaction)
		.await
		.wrap_internal_err("rejecting project after technical review")?;
		ThreadMessageBuilder {
			author_id: Some(user.id.into()),
			body: MessageBody::StatusChange {
				new_status: ProjectStatus::Rejected,
				old_status: ProjectStatus::from_string(&status.old_status),
			},
			thread_id: status.thread_id,
			hide_identity: user.role.is_mod(),
		}
		.insert(&mut transaction)
		.await
		.wrap_internal_err("recording project rejection status change")?;
	}
	transaction
		.commit()
		.await
		.wrap_internal_err("committing technical review verdict")?;
	if submit.verdict == TechReviewVerdict::Unsafe {
		crate::routes::v3::projects::clear_project_cache_and_queue_search(
			&redis,
			&search_state,
			project_id,
			None,
			None,
		)
		.await
		.wrap_api_err("clearing rejected project cache")?;
	}
	Ok(())
}
