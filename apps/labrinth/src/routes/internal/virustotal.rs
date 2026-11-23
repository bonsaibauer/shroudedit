use actix_web::{HttpRequest, post, web};
use eyre::eyre;
use xredis::RedisPool;

use crate::{
	auth::check_is_moderator_from_headers,
	database::{PgPool, models::DBFileId},
	models::{ids::FileId, pats::Scopes},
	queue::virustotal_scan,
	routes::ApiError,
	util::kafka::KafkaClientState,
};

pub fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(run_file_scan);
}

/// Queue a fresh VirusTotal scan for one file.
#[utoipa::path(
	context_path = "/virustotal",
	tag = "virustotal",
	security(("bearer_auth" = [])),
	responses((status = NO_CONTENT), (status = NOT_FOUND))
)]
#[post("/virustotal/run/{file_id}")]
pub async fn run_file_scan(
	req: HttpRequest,
	pool: web::Data<PgPool>,
	redis: web::Data<RedisPool>,
	session_queue: web::Data<crate::queue::session::AuthQueue>,
	kafka_client: web::Data<KafkaClientState>,
	path: web::Path<(FileId,)>,
) -> Result<(), ApiError> {
	check_is_moderator_from_headers(
		&req,
		&**pool,
		&redis,
		&session_queue,
		Scopes::PROJECT_WRITE,
	)
	.await
	.map_err(|error| {
		ApiError::Auth(eyre!("authenticating API request: {error}"))
	})?;

	let file_id = DBFileId::from(path.into_inner().0);
	let file_exists = sqlx::query_scalar!(
		"SELECT EXISTS(SELECT 1 FROM files WHERE id = $1) AS \"exists!\"",
		file_id.0,
	)
	.fetch_one(&**pool)
	.await
	.map_err(|error| {
		ApiError::Internal(eyre!("checking scan file existence: {error}"))
	})?;
	if !file_exists {
		return Err(ApiError::NotFound(eyre!("file not found")));
	}

	let mut transaction = pool.begin().await.map_err(|error| {
		ApiError::Internal(eyre!("beginning scan enqueue transaction: {error}"))
	})?;
	virustotal_scan::force_enqueue_file(
		&mut transaction,
		&kafka_client,
		file_id,
	)
	.await
	.map_err(|error| {
		ApiError::Internal(eyre!("queueing VirusTotal scan: {error}"))
	})?;
	transaction.commit().await.map_err(|error| {
		ApiError::Internal(eyre!(
			"committing scan enqueue transaction: {error}"
		))
	})?;
	Ok(())
}
