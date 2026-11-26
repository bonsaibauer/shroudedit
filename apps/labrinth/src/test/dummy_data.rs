use std::io::{Cursor, Write};

use crate::models::ids::ProjectId;
use crate::models::{
	oauth_clients::OAuthClient,
	organizations::Organization,
	projects::{Project, Version},
};
use crate::test::asserts::assert_status;
use crate::test::{api_common::Api, api_v3, database::USER_USER_PAT};
use actix_http::StatusCode;
use actix_web::test::{self, TestRequest};
use serde_json::json;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

use super::{
	api_common::{ApiProject, AppendsOptionalPat, request_data::ImageData},
	api_v3::ApiV3,
	database::TemporaryDatabase,
};

use super::{database::USER_USER_ID, get_json_val_str};

pub const DUMMY_DATA_UPDATE: i64 = 11;

pub const DUMMY_CATEGORIES: &[&str] = &[
	"combat",
	"decoration",
	"economy",
	"food",
	"magic",
	"mobs",
	"optimization",
];

pub const DUMMY_OAUTH_CLIENT_ALPHA_SECRET: &str = "abcdefghijklmnopqrstuvwxyz";

#[derive(Clone)]
pub enum TestFile {
	DummyProjectAlpha,
	DummyProjectBeta,
	BasicZip,
	BasicMod,
	BasicModDifferent,
	// Randomly generates a valid ShroudForge ZIP with a random hash.
	// Unlike the other dummy mod files, this one is not a static file.
	// and BasicModRandom.bytes() will return a different file each time.
	BasicModRandom { filename: String, bytes: Vec<u8> },
	BasicModpackRandom { filename: String, bytes: Vec<u8> },
}

impl TestFile {
	pub fn build_random_mod_zip() -> Self {
		let filename = format!("random-mod-{}.zip", rand::random::<u64>());

		// Create a simulated zip file
		let mut cursor = Cursor::new(Vec::new());
		{
			let mut zip = ZipWriter::new(&mut cursor);
			zip.start_file(
				"content.txt",
				FileOptions::<()>::default()
					.compression_method(CompressionMethod::Stored),
			)
			.unwrap();
			zip.write_all(b"Enshrouded mod test payload").unwrap();

			zip.finish().unwrap();
		}
		let bytes = cursor.into_inner();

		TestFile::BasicModRandom { filename, bytes }
	}

	pub fn build_random_sfpack(slug: &str, version: &str) -> Self {
		let filename =
			format!("random-modpack-{}.sfpack", rand::random::<u64>());

		let modpack_json = serde_json::json!({
			"id": slug.to_ascii_lowercase(),
			"name": filename,
			"version": version,
			"gameVersions": ["0.1.0"],
			"mods": [{ "path": "mods/test-mod" }],
		})
		.to_string();
		let mod_json = serde_json::json!({
			"id": "test-mod",
			"name": "Test Mod",
			"version": "1.0.0",
		})
		.to_string();

		// Create a simulated zip file
		let mut cursor = Cursor::new(Vec::new());
		{
			let mut zip = ZipWriter::new(&mut cursor);
			zip.start_file(
				"modpack.json",
				FileOptions::<()>::default()
					.compression_method(CompressionMethod::Stored),
			)
			.unwrap();
			zip.write_all(modpack_json.as_bytes()).unwrap();
			zip.start_file(
				"mods/test-mod/mod.json",
				FileOptions::<()>::default()
					.compression_method(CompressionMethod::Stored),
			)
			.unwrap();
			zip.write_all(mod_json.as_bytes()).unwrap();
			zip.finish().unwrap();
		}
		let bytes = cursor.into_inner();

		TestFile::BasicModpackRandom { filename, bytes }
	}
}

#[derive(Clone)]
pub enum DummyImage {
	SmallIcon, // 200x200
}

#[derive(Clone)]
pub struct DummyData {
	/// Alpha project:
	/// This is a dummy project created by USER user.
	/// It's approved, listed, and visible to the public.
	pub project_alpha: DummyProjectAlpha,

	/// Beta project:
	/// This is a dummy project created by USER user.
	/// It's not approved, unlisted, and not visible to the public.
	pub project_beta: DummyProjectBeta,

	/// Zeta organization:
	/// This is a dummy organization created by USER user.
	/// There are no projects in it.
	pub organization_zeta: DummyOrganizationZeta,

	/// Alpha OAuth Client:
	/// This is a dummy OAuth client created by USER user.
	///
	/// All scopes are included in its max scopes
	///
	/// It has one valid redirect URI
	pub oauth_client_alpha: DummyOAuthClientAlpha,
}

impl DummyData {
	pub fn new(
		project_alpha: Project,
		project_alpha_version: Version,
		project_beta: Project,
		project_beta_version: Version,
		organization_zeta: Organization,
		oauth_client_alpha: OAuthClient,
	) -> Self {
		DummyData {
			project_alpha: DummyProjectAlpha {
				team_id: project_alpha.team_id.to_string(),
				project_id: project_alpha.id.to_string(),
				project_slug: project_alpha.slug.unwrap(),
				project_id_parsed: project_alpha.id,
				version_id: project_alpha_version.id.to_string(),
				thread_id: project_alpha.thread_id.to_string(),
				file_hash: project_alpha_version.files[0].hashes["sha1"]
					.clone(),
			},

			project_beta: DummyProjectBeta {
				team_id: project_beta.team_id.to_string(),
				project_id: project_beta.id.to_string(),
				project_slug: project_beta.slug.unwrap(),
				project_id_parsed: project_beta.id,
				version_id: project_beta_version.id.to_string(),
				thread_id: project_beta.thread_id.to_string(),
				file_hash: project_beta_version.files[0].hashes["sha1"].clone(),
			},

			organization_zeta: DummyOrganizationZeta {
				organization_id: organization_zeta.id.to_string(),
				team_id: organization_zeta.team_id.to_string(),
				organization_slug: organization_zeta.slug,
			},

			oauth_client_alpha: DummyOAuthClientAlpha {
				client_id: get_json_val_str(oauth_client_alpha.id),
				client_secret: DUMMY_OAUTH_CLIENT_ALPHA_SECRET.to_string(),
				valid_redirect_uri: oauth_client_alpha
					.redirect_uris
					.first()
					.unwrap()
					.uri
					.clone(),
			},
		}
	}
}

#[derive(Clone)]
pub struct DummyProjectAlpha {
	pub project_id: String,
	pub project_slug: String,
	pub project_id_parsed: ProjectId,
	pub version_id: String,
	pub thread_id: String,
	pub file_hash: String,
	pub team_id: String,
}

#[derive(Clone)]
pub struct DummyProjectBeta {
	pub project_id: String,
	pub project_slug: String,
	pub project_id_parsed: ProjectId,
	pub version_id: String,
	pub thread_id: String,
	pub file_hash: String,
	pub team_id: String,
}

#[derive(Clone)]
pub struct DummyOrganizationZeta {
	pub organization_id: String,
	pub organization_slug: String,
	pub team_id: String,
}

#[derive(Clone)]
pub struct DummyOAuthClientAlpha {
	pub client_id: String,
	pub client_secret: String,
	pub valid_redirect_uri: String,
}

pub async fn add_dummy_data(api: &ApiV3, db: TemporaryDatabase) -> DummyData {
	// Adds basic dummy data to the database directly with sql (user, pats)
	let pool = &db.pool.clone();

	crate::test::db::add_dummy_data(pool).await.unwrap();

	let (alpha_project, alpha_version) = add_project_alpha(api).await;
	let (beta_project, beta_version) = add_project_beta(api).await;

	let zeta_organization = add_organization_zeta(api).await;

	let oauth_client_alpha = get_oauth_client_alpha(api).await;

	sqlx::raw_sql(&format!(
		"INSERT INTO dummy_data (update_id) VALUES ({DUMMY_DATA_UPDATE})"
	))
	.execute(pool)
	.await
	.unwrap();

	DummyData::new(
		alpha_project,
		alpha_version,
		beta_project,
		beta_version,
		zeta_organization,
		oauth_client_alpha,
	)
}

pub async fn get_dummy_data(api: &ApiV3) -> DummyData {
	let (alpha_project, alpha_version) = get_project_alpha(api).await;
	let (beta_project, beta_version) = get_project_beta(api).await;

	let zeta_organization = get_organization_zeta(api).await;

	let oauth_client_alpha = get_oauth_client_alpha(api).await;

	DummyData::new(
		alpha_project,
		alpha_version,
		beta_project,
		beta_version,
		zeta_organization,
		oauth_client_alpha,
	)
}

pub async fn add_project_alpha(api: &ApiV3) -> (Project, Version) {
	let (project, versions) = api
		.add_public_project(
			"alpha",
			Some(TestFile::DummyProjectAlpha),
			None,
			USER_USER_PAT,
		)
		.await;
	let alpha_project = api
		.get_project_deserialized(
			project.id.to_string().as_str(),
			USER_USER_PAT,
		)
		.await;
	let alpha_version = api
		.get_version_deserialized(
			&versions.into_iter().next().unwrap().id.to_string(),
			USER_USER_PAT,
		)
		.await;
	(alpha_project, alpha_version)
}

pub async fn add_project_beta(api: &ApiV3) -> (Project, Version) {
	// Adds dummy data to the database with sqlx (projects, versions, threads)
	// Generate test project data.
	let jar = TestFile::DummyProjectBeta;
	// TODO: this shouldnt be hardcoded (nor should other similar ones be)

	let modify_json = serde_json::from_value(json!([
        { "op": "add", "path": "/summary", "value": "A dummy project for testing with." },
        { "op": "add", "path": "/description", "value": "This project is not yet approved, but it contains enough descriptive information to satisfy project review validation requirements while remaining available for testing private project behavior." },
        { "op": "add", "path": "/initial_versions/0/status", "value": "unlisted" },
        { "op": "add", "path": "/status", "value": "private" },
        { "op": "add", "path": "/requested_status", "value": "private" },
    ]))
    .unwrap();

	let creation_data = api_v3::request_data::get_public_project_creation_data(
		"beta",
		Some(jar),
		Some(modify_json),
	);
	let resp = api.create_project(creation_data, USER_USER_PAT).await;
	assert_status!(&resp, StatusCode::OK);
	crate::test::db::mark_fixture_project_scanned(resp).await;

	get_project_beta(api).await
}

pub async fn add_organization_zeta(api: &ApiV3) -> Organization {
	// Add an organization.
	let req = TestRequest::post()
		.uri("/v3/organization")
		.append_pat(USER_USER_PAT)
		.set_json(json!({
			"name": "Zeta",
			"slug": "zeta",
			"description": "A dummy organization for testing with."
		}))
		.to_request();
	let resp = api.call(req).await;

	assert_status!(&resp, StatusCode::OK);

	get_organization_zeta(api).await
}

pub async fn get_project_alpha(api: &ApiV3) -> (Project, Version) {
	// Get project
	let req = TestRequest::get()
		.uri("/v3/project/alpha")
		.append_pat(USER_USER_PAT)
		.to_request();
	let resp = api.call(req).await;
	let project: Project = test::read_body_json(resp).await;

	// Get project's versions
	let req = TestRequest::get()
		.uri("/v3/project/alpha/version")
		.append_pat(USER_USER_PAT)
		.to_request();
	let resp = api.call(req).await;
	let versions: Vec<Version> = test::read_body_json(resp).await;
	let version = versions.into_iter().next().unwrap();

	(project, version)
}

pub async fn get_project_beta(api: &ApiV3) -> (Project, Version) {
	// Get project
	let req = TestRequest::get()
		.uri("/v3/project/beta")
		.append_pat(USER_USER_PAT)
		.to_request();
	let resp = api.call(req).await;
	assert_status!(&resp, StatusCode::OK);
	let project: serde_json::Value = test::read_body_json(resp).await;
	let project: Project = serde_json::from_value(project).unwrap();

	// Get project's versions
	let req = TestRequest::get()
		.uri("/v3/project/beta/version")
		.append_pat(USER_USER_PAT)
		.to_request();
	let resp = api.call(req).await;
	assert_status!(&resp, StatusCode::OK);
	let versions: Vec<Version> = test::read_body_json(resp).await;
	let version = versions.into_iter().next().unwrap();

	(project, version)
}

pub async fn get_organization_zeta(api: &ApiV3) -> Organization {
	// Get organization
	let req = TestRequest::get()
		.uri("/v3/organization/zeta")
		.append_pat(USER_USER_PAT)
		.to_request();
	let resp = api.call(req).await;
	let organization: Organization = test::read_body_json(resp).await;

	organization
}

pub async fn get_oauth_client_alpha(api: &ApiV3) -> OAuthClient {
	let oauth_clients = api
		.get_user_oauth_clients(USER_USER_ID, USER_USER_PAT)
		.await;
	oauth_clients.into_iter().next().unwrap()
}

impl TestFile {
	pub fn filename(&self) -> String {
		match self {
			TestFile::DummyProjectAlpha => "dummy-project-alpha.zip",
			TestFile::DummyProjectBeta => "dummy-project-beta.zip",
			TestFile::BasicZip => "simple-zip.zip",
			TestFile::BasicMod => "basic-mod.zip",
			TestFile::BasicModDifferent => "basic-mod-different.zip",
			TestFile::BasicModRandom { filename, .. } => filename,
			TestFile::BasicModpackRandom { filename, .. } => filename,
		}
		.to_string()
	}

	pub fn bytes(&self) -> Vec<u8> {
		match self {
			TestFile::DummyProjectAlpha => {
				include_bytes!("../../tests/files/dummy-project-alpha.zip")
					.to_vec()
			}
			TestFile::DummyProjectBeta => {
				include_bytes!("../../tests/files/dummy-project-beta.zip")
					.to_vec()
			}
			TestFile::BasicMod => {
				include_bytes!("../../tests/files/basic-mod.zip").to_vec()
			}
			TestFile::BasicZip => {
				include_bytes!("../../tests/files/simple-zip.zip").to_vec()
			}
			TestFile::BasicModDifferent => {
				include_bytes!("../../tests/files/basic-mod-different.zip")
					.to_vec()
			}
			TestFile::BasicModRandom { bytes, .. } => bytes.clone(),
			TestFile::BasicModpackRandom { bytes, .. } => bytes.clone(),
		}
	}

	pub fn project_type(&self) -> String {
		match self {
			TestFile::DummyProjectAlpha => "mod",
			TestFile::DummyProjectBeta => "mod",
			TestFile::BasicMod => "mod",
			TestFile::BasicModDifferent => "mod",
			TestFile::BasicModRandom { .. } => "mod",

			TestFile::BasicZip => "mod",

			TestFile::BasicModpackRandom { .. } => "modpack",
		}
		.to_string()
	}

	pub fn content_type(&self) -> Option<String> {
		match self {
			TestFile::DummyProjectAlpha => Some("application/zip"),
			TestFile::DummyProjectBeta => Some("application/zip"),
			TestFile::BasicMod => Some("application/zip"),
			TestFile::BasicModDifferent => Some("application/zip"),
			TestFile::BasicModRandom { .. } => Some("application/zip"),

			TestFile::BasicZip => Some("application/zip"),

			TestFile::BasicModpackRandom { .. } => {
				Some("application/x-shroudforge-modpack+zip")
			}
		}
		.map(|s| s.to_string())
	}
}

impl DummyImage {
	pub fn filename(&self) -> String {
		match self {
			DummyImage::SmallIcon => "200x200.png",
		}
		.to_string()
	}

	pub fn extension(&self) -> String {
		match self {
			DummyImage::SmallIcon => "png",
		}
		.to_string()
	}

	pub fn bytes(&self) -> Vec<u8> {
		match self {
			DummyImage::SmallIcon => {
				include_bytes!("../../tests/files/200x200.png").to_vec()
			}
		}
	}

	pub fn get_icon_data(&self) -> ImageData {
		ImageData {
			filename: self.filename(),
			extension: self.extension(),
			icon: self.bytes(),
		}
	}
}
