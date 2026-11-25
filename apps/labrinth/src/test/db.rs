use crate::database::Executor;
use actix_web::{dev::ServiceResponse, test, web};
use eyre::{Context, Result};
use serde::Deserialize;
use serde_json::json;

use crate::database::{
	PgPool,
	models::{DBProjectId, DBUserId},
};
use crate::models::ids::ProjectId;

/// Static personal access token for use in [`AppendPat`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pat(pub &'static str);

/// Dummy [`Pat`]s.
#[allow(missing_docs, reason = "self-explanatory")]
pub mod pat {
	use super::Pat;

	pub const ADMIN: Pat = Pat("mrp_patadmin");
	pub const MODERATOR: Pat = Pat("mrp_patmoderator");
	pub const USER: Pat = Pat("mrp_patuser");
	pub const FRIEND: Pat = Pat("mrp_patfriend");
	pub const ENEMY: Pat = Pat("mrp_patenemy");
}

/// See [`AppendPat::append_pat`].
pub trait AppendPat {
	/// Appends a [`Pat`] authorization token to an
	/// [`actix_web::test::TestRequest`].
	#[must_use]
	fn append_pat(self, pat: Pat) -> Self;
}

impl AppendPat for actix_web::test::TestRequest {
	fn append_pat(self, pat: Pat) -> Self {
		self.append_header(("Authorization", pat.0))
	}
}

/// Mark a user's email as verified without using the email verification flow.
///
/// # Errors
///
/// Errors if the user does not exist or could not be updated.
pub async fn mark_email_verified(db: &PgPool, user_id: DBUserId) -> Result<()> {
	let result = sqlx::query!(
		"UPDATE users SET email_verified = TRUE WHERE id = $1",
		user_id.0,
	)
	.execute(db)
	.await
	.wrap_err("failed to mark user email as verified")?;

	eyre::ensure!(result.rows_affected() == 1, "user does not exist");

	Ok(())
}

/// Record a successful scan for the files uploaded to create a public test fixture.
pub(super) async fn mark_fixture_project_scanned(response: ServiceResponse) {
	#[derive(Deserialize)]
	struct CreatedProject {
		id: ProjectId,
	}

	let pool = response
		.request()
		.app_data::<web::Data<PgPool>>()
		.expect("missing test database pool")
		.clone();
	let project: CreatedProject = test::read_body_json(response).await;
	let project_id = DBProjectId::from(project.id);

	sqlx::query!(
		r#"
		UPDATE virustotal_file_scans AS scan
		SET status = 'completed',
			analysis_id = 'test-fixture-' || scan.file_id::text,
			analysis = $2,
			submitted_at = NOW(),
			checked_at = NOW(),
			updated_at = NOW()
		FROM files
		JOIN versions ON versions.id = files.version_id
		WHERE scan.file_id = files.id AND versions.mod_id = $1
		"#,
		project_id.0,
		json!({
			"stats": { "malicious": 0, "suspicious": 0 },
			"results": { "test-fixture": { "category": "undetected" } },
		}),
	)
	.execute(pool.get_ref())
	.await
	.expect("failed to record successful malware scans for test fixture");
}

/// Dummy [`DBUserId`]s.
///
/// [`DBUserId`]: crate::database::models::DBUserId
pub mod user_id {
	use crate::database::models::DBUserId;

	pub const ADMIN: DBUserId = DBUserId(1);
	pub const MODERATOR: DBUserId = DBUserId(2);
	pub const USER: DBUserId = DBUserId(3);
	pub const FRIEND: DBUserId = DBUserId(4);
	pub const ENEMY: DBUserId = DBUserId(5);
}

/// Initialize a database with dummy fixture data.
///
/// # Errors
///
/// Errors if the fixture could not be applied.
pub async fn add_dummy_data(db: &PgPool) -> Result<()> {
	db.execute(
		include_str!("../../fixtures/dummy_data.sql")
			//
			.replace("{{user_id::ADMIN}}", &user_id::ADMIN.0.to_string())
			.replace(
				"{{user_id::MODERATOR}}",
				&user_id::MODERATOR.0.to_string(),
			)
			.replace("{{user_id::USER}}", &user_id::USER.0.to_string())
			.replace("{{user_id::FRIEND}}", &user_id::FRIEND.0.to_string())
			.replace("{{user_id::ENEMY}}", &user_id::ENEMY.0.to_string())
			//
			.replace("{{pat::ADMIN}}", pat::ADMIN.0)
			.replace("{{pat::MODERATOR}}", pat::MODERATOR.0)
			.replace("{{pat::USER}}", pat::USER.0)
			.replace("{{pat::FRIEND}}", pat::FRIEND.0)
			.replace("{{pat::ENEMY}}", pat::ENEMY.0)
			//
			.replace(
				"{{all_scopes}}",
				&crate::models::pats::Scopes::all().bits().to_string(),
			)
			.as_str(),
	)
	.await
	.wrap_err("failed to add dummy data")?;

	for user_id in [
		user_id::ADMIN,
		user_id::MODERATOR,
		user_id::USER,
		user_id::FRIEND,
		user_id::ENEMY,
	] {
		mark_email_verified(db, user_id).await?;
	}

	Ok(())
}
