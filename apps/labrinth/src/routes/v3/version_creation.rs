use super::project_creation::{CreateError, UploadedFile};
use crate::auth::get_user_from_headers;
use crate::database::PgPool;
use crate::database::PgTransaction;
use crate::database::models::loader_fields::{
	LoaderField, LoaderFieldEnumValue, VersionField,
};
use crate::database::models::notification_item::NotificationBuilder;
use crate::database::models::version_item::{
	DependencyBuilder, VersionBuilder, VersionFileBuilder,
};
use crate::database::models::{self, DBOrganization, image_item};
use crate::env::ENV;
use crate::file_hosting::{FileHost, FileHostPublicity};
use crate::models::exp;
use crate::models::ids::{ImageId, ProjectId, VersionId};
use crate::models::images::{Image, ImageContext};
use crate::models::notifications::NotificationBody;
use crate::models::pats::Scopes;
use crate::models::projects::{
	Dependency, FileType, Loader, Version, VersionFile, VersionStatus,
	VersionType,
};
use crate::models::projects::skip_nulls;
use crate::models::teams::ProjectPermissions;
use crate::models::v3::user_limits::UserLimits;
use crate::queue::session::AuthQueue;
use crate::search::SearchState;
use crate::util::kafka::KafkaClientState;
use crate::util::routes::read_from_field;
use crate::util::validate::validation_errors_to_string;
use crate::validate::{ValidationResult, validate_file};
use actix_multipart::{Field, Multipart};
use actix_web::web::Data;
use actix_web::{HttpRequest, HttpResponse, post, web};
use chrono::Utc;
use futures::stream::StreamExt;
use hex::ToHex;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::collections::{HashMap, HashSet};
use validator::Validate;
use xredis::RedisPool;

fn default_requested_status() -> VersionStatus {
	VersionStatus::Listed
}

const SCHEMATIC_VERSION_FIELDS: [&str; 6] = [
	"schematic_format_version",
	"world_editor_version",
	"schematic_width",
	"schematic_height",
	"schematic_depth",
	"schematic_installation",
];

#[derive(Serialize, Deserialize, Validate, Clone)]
pub struct InitialVersionData {
	#[serde(alias = "mod_id")]
	pub project_id: Option<ProjectId>,
	#[validate(length(min = 1, max = 256))]
	pub file_parts: Vec<String>,
	#[validate(
        length(min = 1, max = 32),
        regex(path = *crate::util::validate::RE_URL_SAFE_RELAXED)
    )]
	pub version_number: String,
	#[validate(
		length(min = 1, max = 120),
		custom(function = "crate::util::validate::validate_name")
	)]
	#[serde(alias = "name")]
	pub version_title: String,
	#[validate(length(max = 65536))]
	#[serde(alias = "changelog")]
	pub version_body: Option<String>,
	#[validate(
		length(min = 0, max = 4096),
		custom(function = "crate::util::validate::validate_deps")
	)]
	pub dependencies: Vec<Dependency>,
	#[serde(alias = "version_type")]
	pub release_channel: VersionType,
	#[validate(length(min = 1))]
	pub loaders: Vec<Loader>,
	pub featured: bool,
	pub primary_file: Option<String>,
	#[serde(default = "default_requested_status")]
	pub status: VersionStatus,
	#[serde(default = "HashMap::new")]
	pub file_types: HashMap<String, Option<FileType>>,
	// Associations to uploaded images in changelog
	#[validate(length(max = 10))]
	#[serde(default)]
	pub uploaded_images: Vec<ImageId>,
	// The ordering relative to other versions
	pub ordering: Option<i32>,

	// Flattened loader fields
	// All other fields are loader-specific VersionFields
	// These are flattened during serialization
	#[serde(deserialize_with = "skip_nulls")]
	#[serde(flatten)]
	pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct InitialFileData {
	#[serde(default = "HashMap::new")]
	pub file_types: HashMap<String, Option<FileType>>,
}

// under `/api/v1/version`
/// Create a version on an existing project.
#[utoipa::path(
	tag = "versions",
	post,
	request_body(
		content(("multipart/form-data")),
		description = "Multipart payload containing `data` and uploaded files"
	),
	responses(
		(status = 200, description = "Expected response to a valid request", body = Version),
		(status = 400, description = "Request was invalid, see given error"),
		(
			status = 401,
			description = "Incorrect token scopes or no authorization to access the requested item(s)"
		)
	),
	security(("bearer_auth" = ["VERSION_CREATE"]))
)]
#[post("/version")]
pub async fn version_create_route(
	req: HttpRequest,
	payload: Multipart,
	client: Data<PgPool>,
	redis: Data<RedisPool>,
	file_host: Data<dyn FileHost>,
	session_queue: Data<AuthQueue>,
	kafka_client: web::Data<KafkaClientState>,
	search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
	version_create(
		req,
		payload,
		client,
		redis,
		file_host,
		session_queue,
		kafka_client,
		search_state,
	)
	.await
}

pub async fn version_create(
	req: HttpRequest,
	mut payload: Multipart,
	client: Data<PgPool>,
	redis: Data<RedisPool>,
	file_host: Data<dyn FileHost>,
	session_queue: Data<AuthQueue>,
	kafka_client: web::Data<KafkaClientState>,
	search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
	let mut transaction = client.begin().await?;
	let mut uploaded_files = Vec::new();

	let result = version_create_inner(
		req,
		&mut payload,
		&mut transaction,
		&redis,
		&**file_host,
		&mut uploaded_files,
		&client,
		&session_queue,
		&kafka_client,
	)
	.await;

	if result.is_err() {
		let undo_result = super::project_creation::undo_uploads(
			&**file_host,
			&uploaded_files,
		)
		.await;
		let rollback_result = transaction.rollback().await;

		undo_result?;
		if let Err(e) = rollback_result {
			return Err(e.into());
		}
	} else if let Ok((_, project_id, version_id)) = &result {
		transaction.commit().await?;
		models::DBProject::clear_cache(*project_id, None, Some(true), &redis)
			.await?;
		search_state
			.queue
			.push_version_changes(
				(*project_id).into(),
				[VersionId::from(*version_id)],
			)
			.await;
	}

	result.map(|(response, _, _)| response)
}

#[allow(clippy::too_many_arguments)]
async fn version_create_inner(
	req: HttpRequest,
	payload: &mut Multipart,
	transaction: &mut PgTransaction<'_>,
	redis: &RedisPool,
	file_host: &dyn FileHost,
	uploaded_files: &mut Vec<UploadedFile>,
	pool: &PgPool,
	session_queue: &AuthQueue,
	kafka_client: &KafkaClientState,
) -> Result<(HttpResponse, models::DBProjectId, models::DBVersionId), CreateError>
{
	let mut initial_version_data = None;
	let mut version_builder = None;
	let mut selected_loaders = None;
	let mut project_type = None::<String>;
	let mut project_slug: Option<String> = None;

	let user = get_user_from_headers(
		&req,
		pool,
		redis,
		session_queue,
		Scopes::VERSION_CREATE,
	)
	.await?
	.1;

	let mut error = None;
	while let Some(item) = payload.next().await {
		let mut field: Field = item?;

		if error.is_some() {
			continue;
		}

		let result = async {
            let content_disposition = field.content_disposition().unwrap().clone();
            let name = content_disposition.get_name().ok_or_else(|| {
                CreateError::MissingValueError("Missing content name".to_string())
            })?;

            if name == "data" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk?);
                }

                let version_create_data: InitialVersionData = serde_json::from_slice(&data)?;
                initial_version_data = Some(version_create_data);
                let version_create_data = initial_version_data.as_ref().unwrap();
                if version_create_data.project_id.is_none() {
                    return Err(CreateError::MissingValueError(
                        "Missing project id".to_string(),
                    ));
                }

                version_create_data.validate().map_err(|err| {
                    CreateError::ValidationError(validation_errors_to_string(err, None))
                })?;

                if !version_create_data.status.can_be_requested() {
                    return Err(CreateError::InvalidInput(
                        "Status specified cannot be requested".to_string(),
                    ));
                }

                let project_id: models::DBProjectId = version_create_data.project_id.unwrap().into();

                // Ensure that the project this version is being added to exists
				let project = models::DBProject::get_id(project_id, &mut *transaction, redis)
					.await?
					.ok_or_else(|| {
						CreateError::InvalidInput(
							"An invalid project id was supplied".to_string(),
						)
					})?;
				let is_schematic_project = project.inner.project_type == "schematic";
				project_type = Some(project.inner.project_type.clone());
                project_slug = project.inner.slug.clone();

                // Check that the user creating this version is a team member
                // of the project the version is being added to.
                let team_member = models::DBTeamMember::get_from_user_id_project(
                    project_id,
                    user.id.into(),
                    false,
                    &mut *transaction,
                )
                .await?;

                // Get organization attached, if exists, and the member project permissions
                let organization = models::DBOrganization::get_associated_organization_project_id(
                    project_id,
                    &mut *transaction,
                )
                .await?;

                let organization_team_member = if let Some(organization) = &organization {
                    models::DBTeamMember::get_from_user_id(
                        organization.team_id,
                        user.id.into(),
                        &mut *transaction,
                    )
                    .await?
                } else {
                    None
                };

                let permissions = ProjectPermissions::get_permissions_by_role(
                    &user.role,
                    &team_member,
                    &organization_team_member,
                )
                .unwrap_or_default();

                if !permissions.contains(ProjectPermissions::UPLOAD_VERSION) {
                    return Err(CreateError::CustomAuthenticationError(
                        "You don't have permission to upload this version!".to_string(),
                    ));
                }

                let project_version_limits =
                    UserLimits::get_for_versions_per_project(
                        &user,
                        project_id,
                        pool,
                    )
                    .await?;
                if project_version_limits.current >= project_version_limits.max {
                    return Err(CreateError::ProjectVersionLimitReached);
                }

                let daily_version_limits =
                    UserLimits::get_for_versions_per_day(
                        &user,
                        Utc::now(),
                        pool,
                    )
                    .await?;
                if daily_version_limits.current >= daily_version_limits.max {
                    return Err(CreateError::DailyVersionLimitReached);
                }

                let version_id: VersionId = models::generate_version_id(transaction).await?.into();

                let all_loaders =
                    models::loader_fields::Loader::list(&mut *transaction, redis).await?;
                let loaders = version_create_data
                    .loaders
                    .iter()
                    .map(|x| {
                        all_loaders
                            .iter()
                            .find(|y| y.loader == x.0)
                            .cloned()
                            .ok_or_else(|| CreateError::InvalidLoader(x.0.clone()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
				if loaders.iter().any(|loader| !loader.can_publish_project_type(&project.inner.project_type)) {
					return Err(CreateError::InvalidInput("A loader does not support this project's type".to_string()));
				}
                selected_loaders = Some(loaders.clone());
                let loader_ids: Vec<models::LoaderId> = loaders.iter().map(|y| y.id).collect_vec();

                let loader_fields =
                    LoaderField::get_fields(&loader_ids, &mut *transaction, redis).await?;
                let mut loader_field_enum_values = LoaderFieldEnumValue::list_many_loader_fields(
                    &loader_fields,
                    &mut *transaction,
                    redis,
                )
                .await?;
                let version_fields = try_create_version_fields(
                    version_id,
                    &version_create_data.fields,
                    &loader_fields,
                    &mut loader_field_enum_values,
					is_schematic_project,
                )?;

                let dependencies = version_create_data
                    .dependencies
                    .iter()
                    .map(|d| models::version_item::DependencyBuilder {
                        version_id: d.version_id.map(|x| x.into()),
                        project_id: d.project_id.map(|x| x.into()),
                        dependency_type: d.dependency_type.to_string(),
                        file_name: None,
                    })
                    .collect::<Vec<_>>();

                version_builder = Some(VersionBuilder {
                    version_id: version_id.into(),
                    project_id,
                    author_id: user.id.into(),
                    name: version_create_data.version_title.clone(),
                    version_number: version_create_data.version_number.clone(),
                    changelog: version_create_data.version_body.clone().unwrap_or_default(),
                    files: Vec::new(),
                    dependencies,
                    loaders: loader_ids,
                    version_fields,
                    version_type: version_create_data.release_channel.to_string(),
                    featured: version_create_data.featured,
                    status: version_create_data.status,
                    requested_status: None,
                    ordering: version_create_data.ordering,
                    components: exp::VersionSerial::default(),
                });

                return Ok(());
            }

            let version = version_builder.as_mut().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;
            let loaders = selected_loaders.as_ref().ok_or_else(|| {
                CreateError::InvalidInput(String::from("`data` field must come before file fields"))
            })?;
            let loaders = loaders
                .iter()
                .map(|x| Loader(x.loader.clone()))
                .collect::<Vec<_>>();

            let version_data = initial_version_data
                .clone()
                .ok_or_else(|| CreateError::InvalidInput("`data` field is required".to_string()))?;

            let existing_file_names = version.files.iter().map(|x| x.filename.clone()).collect();

            upload_file(
                &mut field,
                file_host,
                version_data.file_parts.len(),
                uploaded_files,
                &mut version.files,
                &mut version.dependencies,
                &content_disposition,
                version.project_id.into(),
                version.version_id.into(),
                &version.version_fields,
                loaders,
                version_data.primary_file.is_some(),
                version_data.primary_file.as_deref() == Some(name),
                version_data.file_types.get(name).copied().flatten(),
                existing_file_names,
				project_slug.as_deref(),
				&version.version_number,
                transaction,
                redis,
            )
            .await?;

            Ok(())
        }
        .await;

		if result.is_err() {
			error = result.err();
		}
	}

	if let Some(error) = error {
		return Err(error);
	}

	let version_data = initial_version_data.ok_or_else(|| {
		CreateError::InvalidInput("`data` field is required".to_string())
	})?;
	let builder = version_builder.ok_or_else(|| {
		CreateError::InvalidInput("`data` field is required".to_string())
	})?;

	if builder.files.is_empty() {
		return Err(CreateError::InvalidInput(
			"Versions must have at least one file uploaded to them".to_string(),
		));
	}

	use futures::stream::TryStreamExt;

	let users = sqlx::query!(
		"
        SELECT mf.follower_id FROM mod_follows mf
        INNER JOIN users u ON u.id = follower_id
        WHERE mf.mod_id = $1
        ",
		builder.project_id as crate::database::models::ids::DBProjectId
	)
	.fetch(&mut *transaction)
	.map_ok(|m| models::ids::DBUserId(m.follower_id))
	.try_collect::<Vec<models::ids::DBUserId>>()
	.await?;

	let project_id: ProjectId = builder.project_id.into();
	let version_id: VersionId = builder.version_id.into();

	NotificationBuilder {
		body: NotificationBody::ProjectUpdate {
			project_id,
			version_id,
		},
	}
	.insert_many(users, &mut *transaction, redis)
	.await?;

	let loader_structs = selected_loaders.unwrap_or_default();
	let all_games = loader_structs
		.iter()
		.flat_map(|loader| loader.supported_games.clone())
		.collect::<Vec<_>>();

	let response = Version {
		id: builder.version_id.into(),
		project_id: builder.project_id.into(),
		author_id: user.id,
		featured: builder.featured,
		name: builder.name.clone(),
		version_number: builder.version_number.clone(),
		project_types: vec![project_type.expect("project type set during version creation")],
		games: all_games,
		changelog: Some(builder.changelog.clone()),
		date_published: Utc::now(),
		downloads: 0,
		version_type: version_data.release_channel,
		status: builder.status,
		requested_status: builder.requested_status,
		ordering: builder.ordering,
		files: builder
			.files
			.iter()
			.map(|file| VersionFile {
				id: None,
				hashes: file
					.hashes
					.iter()
					.map(|hash| {
						(
							hash.algorithm.clone(),
							// This is a hack since the hashes are currently stored as ASCII
							// in the database, but represented here as a Vec<u8>.  At some
							// point we need to change the hash to be the real bytes  in the
							// database and add more processing here.
							String::from_utf8(hash.hash.clone()).unwrap(),
						)
					})
					.collect(),
				url: file.url.clone(),
				filename: file.filename.clone(),
				primary: file.primary,
				size: file.size,
				file_type: file.file_type,
				malware: crate::models::projects::FileMalwareScan::default(),
			})
			.collect::<Vec<_>>(),
		dependencies: version_data.dependencies,
		loaders: version_data.loaders,
		fields: version_data.fields,
		components: exp::VersionQuery::default(),
		files_missing_attribution: Vec::new(),
	};

	let project_id = builder.project_id;
	builder
		.insert(transaction, redis, file_host, kafka_client)
		.await?;

	for image_id in version_data.uploaded_images {
		if let Some(db_image) =
			image_item::DBImage::get(image_id.into(), &mut *transaction, redis)
				.await?
		{
			let image: Image = db_image.into();
			if !matches!(image.context, ImageContext::Report { .. })
				|| image.context.inner_id().is_some()
			{
				return Err(CreateError::InvalidInput(format!(
					"Image {image_id} is not unused and in the 'version' context"
				)));
			}

			sqlx::query!(
				"
                UPDATE uploaded_images
                SET version_id = $1
                WHERE id = $2
                ",
				version_id.0 as i64,
				image_id.0 as i64
			)
			.execute(&mut *transaction)
			.await?;

			image_item::DBImage::clear_cache(image.id.into(), redis).await?;
		} else {
			return Err(CreateError::InvalidInput(format!(
				"Image {image_id} does not exist"
			)));
		}
	}

	Ok((
		HttpResponse::Ok().json(response),
		project_id,
		models::DBVersionId::from(version_id),
	))
}

/// Add files to an existing version.
#[utoipa::path(
	tag = "versions",
	post,
	params(
		("version_id" = VersionId, Path, description = "The ID of the version")
	),
	request_body(
		content(("multipart/form-data")),
		description = "Multipart payload containing files to upload"
	),
	responses(
		(status = NO_CONTENT, description = "Expected response to a valid request"),
		(
			status = 401,
			description = "Incorrect token scopes or no authorization to access the requested item(s)"
		),
		(
			status = 404,
			description = "The requested item(s) were not found or no authorization to access the requested item(s)"
		)
	),
	security(("bearer_auth" = ["VERSION_WRITE"]))
)]
#[post("/version/{version_id}/file")]
pub async fn upload_file_to_version_route(
	req: HttpRequest,
	url_data: web::Path<(VersionId,)>,
	payload: Multipart,
	client: Data<PgPool>,
	redis: Data<RedisPool>,
	file_host: Data<dyn FileHost>,
	session_queue: web::Data<AuthQueue>,
	kafka_client: web::Data<KafkaClientState>,
	search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
	upload_file_to_version(
		req,
		url_data,
		payload,
		client,
		redis,
		file_host,
		session_queue,
		kafka_client,
		search_state,
	)
	.await
}

pub async fn upload_file_to_version(
	req: HttpRequest,
	url_data: web::Path<(VersionId,)>,
	mut payload: Multipart,
	client: Data<PgPool>,
	redis: Data<RedisPool>,
	file_host: Data<dyn FileHost>,
	session_queue: web::Data<AuthQueue>,
	kafka_client: web::Data<KafkaClientState>,
	search_state: Data<SearchState>,
) -> Result<HttpResponse, CreateError> {
	let mut transaction = client.begin().await?;
	let mut uploaded_files = Vec::new();

	let version_id = url_data.into_inner().0;
	let db_version_id = models::DBVersionId::from(version_id);

	let result = upload_file_to_version_inner(
		req,
		&mut payload,
		client.clone(),
		&mut transaction,
		redis.clone(),
		&**file_host,
		&mut uploaded_files,
		db_version_id,
		&session_queue,
		&kafka_client,
	)
	.await;

	if result.is_err() {
		let undo_result = super::project_creation::undo_uploads(
			&**file_host,
			&uploaded_files,
		)
		.await;
		let rollback_result = transaction.rollback().await;

		undo_result?;
		if let Err(e) = rollback_result {
			return Err(e.into());
		}
	} else if let Ok((_, project_id, version)) = &result {
		transaction.commit().await?;
		models::DBVersion::clear_cache(version, &redis).await?;
		models::DBProject::clear_cache(*project_id, None, Some(true), &redis)
			.await?;
		search_state
			.queue
			.push_version_changes((*project_id).into(), [version_id])
			.await;
	}

	result.map(|(response, _, _)| response)
}

#[allow(clippy::too_many_arguments)]
async fn upload_file_to_version_inner(
	req: HttpRequest,
	payload: &mut Multipart,
	client: Data<PgPool>,
	transaction: &mut PgTransaction<'_>,
	redis: Data<RedisPool>,
	file_host: &dyn FileHost,
	uploaded_files: &mut Vec<UploadedFile>,
	version_id: models::DBVersionId,
	session_queue: &AuthQueue,
	kafka_client: &KafkaClientState,
) -> Result<
	(
		HttpResponse,
		models::DBProjectId,
		models::version_item::VersionQueryResult,
	),
	CreateError,
> {
	let mut initial_file_data: Option<InitialFileData> = None;
	let mut file_builders: Vec<VersionFileBuilder> = Vec::new();

	let user = get_user_from_headers(
		&req,
		&**client,
		&redis,
		session_queue,
		Scopes::VERSION_WRITE,
	)
	.await?
	.1;

	let result = models::DBVersion::get(version_id, &**client, &redis).await?;

	let Some(version) = result else {
		return Err(CreateError::InvalidInput(
			"An invalid version id was supplied".to_string(),
		));
	};

	let all_loaders =
		models::loader_fields::Loader::list(&mut *transaction, &redis).await?;
	let selected_loaders = version
		.loaders
		.iter()
		.map(|x| {
			all_loaders
				.iter()
				.find(|y| &y.loader == x)
				.cloned()
				.ok_or_else(|| CreateError::InvalidLoader(x.clone()))
		})
		.collect::<Result<Vec<_>, _>>()?;

	let project = models::DBProject::get_id(
		version.inner.project_id,
		&mut *transaction,
		&redis,
	)
	.await?
	.ok_or_else(|| CreateError::InvalidInput("An invalid project id was supplied".to_string()))?;
	if selected_loaders.iter().any(|loader| !loader.can_publish_project_type(&project.inner.project_type)) {
		return Err(CreateError::InvalidInput("A loader does not support this project's type".to_string()));
	}

	if !user.role.is_admin() {
		let team_member = models::DBTeamMember::get_from_user_id_project(
			version.inner.project_id,
			user.id.into(),
			false,
			&mut *transaction,
		)
		.await?;

		let organization =
			DBOrganization::get_associated_organization_project_id(
				version.inner.project_id,
				&**client,
			)
			.await?;

		let organization_team_member = if let Some(organization) = &organization
		{
			models::DBTeamMember::get_from_user_id(
				organization.team_id,
				user.id.into(),
				&mut *transaction,
			)
			.await?
		} else {
			None
		};

		let permissions = ProjectPermissions::get_permissions_by_role(
			&user.role,
			&team_member,
			&organization_team_member,
		)
		.unwrap_or_default();

		if !permissions.contains(ProjectPermissions::UPLOAD_VERSION) {
			return Err(CreateError::CustomAuthenticationError(
				"You don't have permission to upload files to this version!"
					.to_string(),
			));
		}
	}

	let project_id = ProjectId(version.inner.project_id.0 as u64);
	let db_project_id = version.inner.project_id;
	let mut error = None;
	while let Some(item) = payload.next().await {
		let mut field: Field = item?;

		if error.is_some() {
			continue;
		}

		let result = async {
			let content_disposition =
				field.content_disposition().unwrap().clone();
			let name = content_disposition.get_name().ok_or_else(|| {
				CreateError::MissingValueError(
					"Missing content name".to_string(),
				)
			})?;

			if name == "data" {
				let mut data = Vec::new();
				while let Some(chunk) = field.next().await {
					data.extend_from_slice(&chunk?);
				}
				let file_data: InitialFileData = serde_json::from_slice(&data)?;

				initial_file_data = Some(file_data);
				return Ok(());
			}

			let file_data = initial_file_data.as_ref().ok_or_else(|| {
				CreateError::InvalidInput(String::from(
					"`data` field must come before file fields",
				))
			})?;

			let loaders = selected_loaders
				.iter()
				.map(|x| Loader(x.loader.clone()))
				.collect::<Vec<_>>();

			let mut dependencies = version
				.dependencies
				.iter()
				.map(|x| DependencyBuilder {
					project_id: x.project_id,
					version_id: x.version_id,
					file_name: x.file_name.clone(),
					dependency_type: x.dependency_type.clone(),
				})
				.collect();

			upload_file(
				&mut field,
				file_host,
				0,
				uploaded_files,
				&mut file_builders,
				&mut dependencies,
				&content_disposition,
				project_id,
				version_id.into(),
				&version.version_fields,
				loaders,
				true,
				false,
				file_data.file_types.get(name).copied().flatten(),
				version.files.iter().map(|x| x.filename.clone()).collect(),
			project.inner.slug.as_deref(),
				&version.inner.version_number,
				transaction,
				&redis,
			)
			.await?;

			Ok(())
		}
		.await;

		if result.is_err() {
			error = result.err();
		}
	}

	if let Some(error) = error {
		return Err(error);
	}

	if file_builders.is_empty() {
		return Err(CreateError::InvalidInput(
			"At least one file must be specified".to_string(),
		));
	} else {
		let project_id = version.inner.project_id;

		for file in file_builders {
			file.insert(
				version_id,
				project_id,
				&mut *transaction,
				&redis,
				file_host,
				kafka_client,
			)
			.await?;
		}
	}

	Ok((HttpResponse::NoContent().body(""), db_project_id, version))
}

// This function is used for adding a file to a version, uploading the initial
// files for a version, and for uploading the initial version files for a project
#[allow(clippy::too_many_arguments)]
pub async fn upload_file(
	field: &mut Field,
	file_host: &dyn FileHost,
	total_files_len: usize,
	uploaded_files: &mut Vec<UploadedFile>,
	version_files: &mut Vec<VersionFileBuilder>,
	dependencies: &mut Vec<DependencyBuilder>,
	content_disposition: &actix_web::http::header::ContentDisposition,
	project_id: ProjectId,
	version_id: VersionId,
	version_fields: &[VersionField],
	loaders: Vec<Loader>,
	ignore_primary: bool,
	force_primary: bool,
	file_type: Option<FileType>,
	other_file_names: Vec<String>,
	expected_project_slug: Option<&str>,
	expected_version_number: &str,
	transaction: &mut PgTransaction<'_>,
	redis: &RedisPool,
) -> Result<(), CreateError> {
	let (file_name, file_extension) = get_name_ext(content_disposition)?;
	let file_extension = file_extension.to_ascii_lowercase();

	if other_file_names.iter().any(|name| name == file_name) {
		return Err(CreateError::InvalidInput(
			"Duplicate files are not allowed to be uploaded to ShroudEdit!"
				.to_string(),
		));
	}

	if !path_util::is_safe_file_name(file_name) {
		return Err(CreateError::InvalidInput(
			"file names must be a single safe path component".to_string(),
		));
	}

	let content_type = crate::util::ext::project_file_type(&file_extension)
		.ok_or_else(|| CreateError::InvalidFileType(file_extension.clone()))?;

	let (max_file_size, file_size_error) = if file_extension == "sfbp" {
		(
			95 * (1 << 20),
			"SFBP blueprint exceeds the maximum of 95 MiB",
		)
	} else {
		(95 * (1 << 20), "project file exceeds the maximum of 95 MiB")
	};
	let data = read_from_field(field, max_file_size, file_size_error).await?;

	let hash = sha1::Sha1::digest(&data).encode_hex::<String>();
	let exists = sqlx::query!(
		"
        SELECT EXISTS(SELECT 1 FROM hashes h
        INNER JOIN files f ON f.id = h.file_id
        INNER JOIN versions v ON v.id = f.version_id
        WHERE h.algorithm = $2 AND h.hash = $1 AND v.mod_id != $3)
        ",
		hash.as_bytes(),
		"sha1",
		project_id.0 as i64
	)
	.fetch_one(&mut *transaction)
	.await?
	.exists
	.unwrap_or(false);

	if exists {
		return Err(CreateError::InvalidInput(
			"Duplicate files are not allowed to be uploaded to ShroudEdit!"
				.to_string(),
		));
	}

	let data = data.freeze();
	if file_extension == "sfpack" {
		if !(force_primary || total_files_len == 1
			|| (!ignore_primary && version_files.iter().all(|file| !file.primary)))
		{
			return Err(CreateError::InvalidInput(".sfpack must be the primary version file".into()));
		}
		let pack = crate::validate::sfpack::validate_package(
			data.clone(), Some(expected_version_number),
		).map_err(CreateError::InvalidInput)?;
		let mut selected_games = version_fields.iter()
			.find_map(|field| crate::database::models::legacy_loader_fields::EnshroudedGameVersion::try_from_version_field(field).ok())
			.unwrap_or_default()
			.into_iter().map(|game| game.version).collect::<Vec<_>>();
		selected_games.sort();
		let mut pack_games = pack.game_versions.clone();
		pack_games.sort();
		if selected_games != pack_games {
			return Err(CreateError::InvalidInput(".sfpack gameVersions must match the selected game versions".into()));
		}
		let version_ids = pack.mods.iter().map(|item| {
			serde_json::from_value::<VersionId>(serde_json::json!(item.version_id))
				.map(models::DBVersionId::from)
				.map_err(|error| CreateError::InvalidInput(format!("Invalid .sfpack version ID: {error}")))
		}).collect::<Result<Vec<_>, _>>()?;
		let versions = models::DBVersion::get_many(&version_ids, &mut *transaction, redis).await?;
		let version_project_ids = versions
			.iter()
			.map(|version| version.inner.project_id)
			.unique()
			.collect::<Vec<_>>();
		let schematic_project_ids = models::DBProject::get_many_ids(
			&version_project_ids,
			&mut *transaction,
			redis,
		)
		.await?
		.into_iter()
		.filter(|project| project.inner.components.schematic.is_some())
		.map(|project| project.inner.id)
		.collect::<HashSet<_>>();
		dependencies.clear();
		for (item, version_id) in pack.mods.iter().zip(&version_ids) {
			let project_id = serde_json::from_value::<ProjectId>(serde_json::json!(item.project_id))
				.map(models::DBProjectId::from)
				.map_err(|error| CreateError::InvalidInput(format!("Invalid .sfpack project ID: {error}")))?;
			let version = versions.iter().find(|version| version.inner.id == *version_id)
				.ok_or_else(|| CreateError::InvalidInput(format!(".sfpack version '{}' is unavailable", item.version_id)))?;
			if version.inner.project_id != project_id
				|| schematic_project_ids.contains(&project_id)
				|| !matches!(version.inner.status, VersionStatus::Listed | VersionStatus::Unlisted | VersionStatus::Archived)
				|| !version.project_types.iter().any(|kind| kind == "mod")
				|| !version.loaders.iter().any(|loader| matches!(loader.as_str(), "shroudforge" | "eml"))
			{
				return Err(CreateError::InvalidInput(format!(".sfpack version '{}' is not a published ShroudForge mod of project '{}'", item.version_id, item.project_id)));
			}
			let file = version.files.iter().find(|file| file.primary).or_else(|| version.files.first())
				.ok_or_else(|| CreateError::InvalidInput(format!(".sfpack version '{}' has no package", item.version_id)))?;
			if !file.filename.to_ascii_lowercase().ends_with(".zip")
				|| !file.hashes.get("sha512").is_some_and(|hash| hash.eq_ignore_ascii_case(&item.sha512))
			{
				return Err(CreateError::InvalidInput(format!(".sfpack checksum does not match primary ZIP of version '{}'", item.version_id)));
			}
			let games = version.version_fields.iter()
				.find_map(|field| crate::database::models::legacy_loader_fields::EnshroudedGameVersion::try_from_version_field(field).ok())
				.unwrap_or_default();
			if !pack.game_versions.iter().all(|game| games.iter().any(|supported| &supported.version == game)) {
				return Err(CreateError::InvalidInput(format!(".sfpack mod '{}' does not support every declared game version", item.project_id)));
			}
			for dependency in version.dependencies.iter().filter(|dependency| dependency.dependency_type == "required") {
				if (dependency.project_id.is_some() || dependency.version_id.is_some())
					&& !pack.mods.iter().any(|candidate| {
						let candidate_project = serde_json::from_value::<ProjectId>(serde_json::json!(candidate.project_id))
							.map(models::DBProjectId::from).ok();
						let candidate_version = serde_json::from_value::<VersionId>(serde_json::json!(candidate.version_id))
							.map(models::DBVersionId::from).ok();
						dependency.project_id.is_none_or(|id| candidate_project == Some(id))
							&& dependency.version_id.is_none_or(|id| candidate_version == Some(id))
					})
				{
					return Err(CreateError::InvalidInput(format!(".sfpack omits a required catalog dependency of version '{}'", item.version_id)));
				}
			}
			dependencies.push(DependencyBuilder {
				project_id: Some(project_id), version_id: Some(*version_id),
				file_name: None, dependency_type: "required".into(),
			});
		}
	}
	if file_extension == "zip"
		&& loaders
			.iter()
			.any(|loader| matches!(loader.0.as_str(), "shroudforge" | "eml"))
	{
		let primary_candidate = force_primary || total_files_len == 1
			|| (!ignore_primary && file_type.is_none() && version_files.iter().all(|file| !file.primary));
		let package = crate::validate::shroudforge::validate_package(
			data.clone(),
			expected_project_slug,
			expected_version_number,
		)
		.map_err(CreateError::InvalidInput)?;
		if primary_candidate && let Some(package) = package {
			// mod.json is the author's dependency list. The catalog links are derived from it.
			dependencies.clear();
			for dependency in package.dependencies {
				if dependency.id == "shroudforge-api" {
					continue;
				}
				let Some(project) = models::DBProject::get(
					&dependency.id,
					&mut *transaction,
					redis,
				)
				.await?
				else {
					continue;
				};
				if project.inner.slug.as_deref() != Some(dependency.id.as_str()) {
					continue;
				}
				let requirement = semver::VersionReq::parse(&dependency.version)
					.map_err(|error| CreateError::InvalidInput(error.to_string()))?;
				let mut versions = models::DBVersion::get_many(
					&project.versions,
					&mut *transaction,
					redis,
				)
				.await?;
				versions.sort_by(|left, right| right.inner.date_published.cmp(&left.inner.date_published));
				let matching = versions.into_iter().find(|version| {
					version.inner.status == VersionStatus::Listed
						&& version.loaders.iter().any(|loader| matches!(loader.as_str(), "shroudforge" | "eml"))
						&& semver::Version::parse(&version.inner.version_number)
							.is_ok_and(|number| requirement.matches(&number))
				});
				if let Some(version) = matching {
					dependencies.push(DependencyBuilder {
						project_id: Some(project.inner.id),
						version_id: Some(version.inner.id),
						file_name: None,
						dependency_type: if dependency.optional { "optional" } else { "required" }.into(),
					});
				}
			}
		}
	}

	let validation_result = validate_file(
		data.clone(),
		file_extension.to_string(),
		loaders.clone(),
		file_type,
		version_fields.to_vec(),
		&mut *transaction,
		redis,
	)
	.await?;

	let primary = (validation_result.is_passed()
		&& version_files.iter().all(|x| !x.primary)
		&& !ignore_primary)
		|| force_primary
		|| total_files_len == 1;

	let file_path_encode = format!(
		"data/{project_id}/versions/{version_id}/{}",
		urlencoding::encode(file_name)
	);
	let file_path =
		format!("data/{project_id}/versions/{version_id}/{file_name}");

	let upload_data = file_host
		.upload_file(content_type, &file_path, FileHostPublicity::Public, data)
		.await?;

	uploaded_files.push(UploadedFile {
		name: file_path,
		publicity: FileHostPublicity::Public,
	});

	let sha1_bytes = upload_data.content_sha1.into_bytes();
	let sha512_bytes = upload_data.content_sha512.into_bytes();

	if version_files.iter().any(|x| {
		x.hashes
			.iter()
			.any(|y| y.hash == sha1_bytes || y.hash == sha512_bytes)
	}) {
		return Err(CreateError::InvalidInput(
			"Duplicate files are not allowed to be uploaded to ShroudEdit!"
				.to_string(),
		));
	}

	if let ValidationResult::Warning(msg) = validation_result
		&& primary
	{
		return Err(CreateError::InvalidInput(msg.to_string()));
	}

	version_files.push(VersionFileBuilder {
		filename: file_name.to_string(),
		url: format!("{}/{file_path_encode}", ENV.CDN_URL),
		hashes: vec![
			models::version_item::HashBuilder {
				algorithm: "sha1".to_string(),
				// This is an invalid cast - the database expects the hash's
				// bytes, but this is the string version.
				hash: sha1_bytes,
			},
			models::version_item::HashBuilder {
				algorithm: "sha512".to_string(),
				// This is an invalid cast - the database expects the hash's
				// bytes, but this is the string version.
				hash: sha512_bytes,
			},
		],
		primary,
		size: upload_data.content_length,
		file_type,
	});

	Ok(())
}

pub fn get_name_ext(
	content_disposition: &actix_web::http::header::ContentDisposition,
) -> Result<(&str, &str), CreateError> {
	let file_name = content_disposition.get_filename().ok_or_else(|| {
		CreateError::MissingValueError("Missing content file name".to_string())
	})?;
	let file_extension = if let Some(last_period) = file_name.rfind('.') {
		file_name.get((last_period + 1)..).unwrap_or("")
	} else {
		return Err(CreateError::MissingValueError(
			"Missing content file extension".to_string(),
		));
	};
	Ok((file_name, file_extension))
}

// Reused functionality between project_creation and version_creation
// Create a list of VersionFields from the fetched data, and check that all mandatory fields are present
pub fn try_create_version_fields(
	version_id: VersionId,
	submitted_fields: &HashMap<String, serde_json::Value>,
	loader_fields: &[LoaderField],
	loader_field_enum_values: &mut HashMap<
		models::LoaderFieldId,
		Vec<LoaderFieldEnumValue>,
	>,
	is_schematic_project: bool,
) -> Result<Vec<VersionField>, CreateError> {
	let mut version_fields = vec![];
	let applicable_loader_fields = loader_fields
		.iter()
		.filter(|loader_field| {
			is_schematic_project
				|| !SCHEMATIC_VERSION_FIELDS
					.contains(&loader_field.field.as_str())
		})
		.cloned()
		.collect::<Vec<_>>();
	let mut remaining_mandatory_loader_fields = applicable_loader_fields
		.iter()
		.filter(|lf| !lf.optional)
		.map(|lf| lf.field.clone())
		.collect::<HashSet<_>>();
	for (key, value) in submitted_fields {
		let loader_field = applicable_loader_fields
			.iter()
			.find(|lf| lf.field.as_str() == key.as_str())
			.ok_or_else(|| {
				CreateError::InvalidInput(format!(
					"Loader field '{key}' does not exist for any loaders supplied,"
				))
			})?;
		remaining_mandatory_loader_fields.remove(&loader_field.field);
		let enum_variants = loader_field_enum_values
			.remove(&loader_field.id)
			.unwrap_or_default();

		let vf: VersionField = VersionField::check_parse(
			version_id.into(),
			loader_field.clone(),
			value.clone(),
			enum_variants,
		)
		.map_err(CreateError::InvalidInput)?;
		version_fields.push(vf);
	}

	if !remaining_mandatory_loader_fields.is_empty() {
		return Err(CreateError::InvalidInput(format!(
			"Missing mandatory loader fields: {}",
			remaining_mandatory_loader_fields.iter().join(", ")
		)));
	}
	Ok(version_fields)
}
