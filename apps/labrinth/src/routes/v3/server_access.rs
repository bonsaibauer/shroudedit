use std::collections::HashMap;

use actix_web::{
    HttpRequest, Responder, get, http::header::CACHE_CONTROL, put, web,
};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use xredis::RedisPool;

use crate::auth::checks::is_visible_project;
use crate::auth::get_user_from_headers;
use crate::database::models::{DBProject, DBProjectId, DBTeamMember};
use crate::database::{PgPool, ReadOnlyPgPool};
use crate::models::exp::enshrouded::{
    EnshroudedPasswordVisibility, EnshroudedServerRole,
};
use crate::models::pats::Scopes;
use crate::models::teams::ProjectPermissions;
use crate::queue::session::AuthQueue;
use crate::routes::ApiError;
use crate::util::error::Context;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_server_access).service(set_server_access);
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicServerPassword {
    pub group_name: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct SetServerAccess {
    #[validate(length(max = 32))]
    #[validate(nested)]
    pub passwords: Vec<PublicServerPasswordInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PublicServerPasswordInput {
    #[validate(length(min = 1, max = 64))]
    pub group_name: String,
    #[validate(length(min = 1, max = 128))]
    pub password: String,
}

#[utoipa::path(
    context_path = "/project",
    tag = "server_access",
    responses((status = OK, body = Vec<PublicServerPassword>))
)]
#[get("/{project_id}/server-access")]
pub async fn get_server_access(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    ro_pool: web::Data<ReadOnlyPgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<impl Responder, ApiError> {
    let project = DBProject::get(&info.into_inner().0, &***ro_pool, &redis)
        .await
        .wrap_internal_err("fetching project")?
        .wrap_not_found_err("resource not found")?;
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_READ,
    )
    .await
    .map(|(_, user)| user)
    .ok();
    if !is_visible_project(&project.inner, &user, &pool, false)
        .await
        .wrap_internal_err("checking project visibility")?
    {
        return Err(ApiError::NotFound(eyre!("resource not found")));
    }

    let public_groups = project
        .inner
        .components
        .enshrouded_server
        .as_ref()
        .map(|server| {
            server
                .user_groups
                .iter()
                .filter(|group| {
                    group.password_visibility
                        == EnshroudedPasswordVisibility::Public
                        && group.role != EnshroudedServerRole::Admin
                        && !group.can_kick_ban
                })
                .map(|group| group.name.trim().to_lowercase())
                .collect::<std::collections::HashSet<_>>()
        })
        .unwrap_or_default();

    let project_id = DBProjectId::from(project.inner.id).0;
    let rows = sqlx::query!(
        r#"
        SELECT group_name, password
        FROM enshrouded_server_public_passwords
        WHERE project_id = $1
        ORDER BY LOWER(group_name)
        "#,
        project_id,
    )
    .fetch_all(&***ro_pool)
    .await
    .wrap_internal_err("fetching public enshrouded server passwords")?;

    Ok(web::Json(
        rows.into_iter()
            .filter(|row| {
                public_groups.contains(&row.group_name.trim().to_lowercase())
            })
            .map(|row| PublicServerPassword {
                group_name: row.group_name,
                password: row.password,
            })
            .collect::<Vec<_>>(),
    )
    .customize()
    .insert_header((CACHE_CONTROL, "no-store")))
}

#[utoipa::path(
    context_path = "/project",
    tag = "server_access",
    request_body = SetServerAccess,
    responses((status = NO_CONTENT))
)]
#[put("/{project_id}/server-access")]
pub async fn set_server_access(
    req: HttpRequest,
    info: web::Path<(String,)>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
    body: web::Json<SetServerAccess>,
) -> Result<(), ApiError> {
    body.validate()
        .map_err(|error| ApiError::Request(eyre!(error)))?;
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::PROJECT_WRITE,
    )
    .await
    .wrap_auth_err("authenticating API request")?
    .1;
    let project = DBProject::get(&info.into_inner().0, &**pool, &redis)
        .await
        .wrap_internal_err("fetching project")?
        .wrap_not_found_err("resource not found")?;
    let (team_member, organization_team_member) =
        DBTeamMember::get_for_project_permissions(
            &project.inner,
            user.id.into(),
            &**pool,
        )
        .await
        .wrap_internal_err("fetching project permissions")?;
    let can_edit = ProjectPermissions::get_permissions_by_role(
        &user.role,
        &team_member,
        &organization_team_member,
    )
    .is_some_and(|permissions| {
        permissions.contains(ProjectPermissions::EDIT_DETAILS)
    });
    if !can_edit {
        return Err(ApiError::Auth(eyre!(
            "you do not have permission to edit this project's server access"
        )));
    }

    let server = project
        .inner
        .components
        .enshrouded_server
        .as_ref()
        .wrap_request_err(
            "project does not have an `enshrouded_server` component",
        )?;
    let groups = server
        .user_groups
        .iter()
        .map(|group| (group.name.trim().to_lowercase(), group))
        .collect::<HashMap<_, _>>();
    let mut names = std::collections::HashSet::new();
    for entry in &body.passwords {
        if entry.group_name.trim().is_empty()
            || entry.password.trim().is_empty()
        {
            return Err(ApiError::Request(eyre!(
                "public password group names and passwords must not be blank"
            )));
        }
        let normalized = entry.group_name.trim().to_lowercase();
        if !names.insert(normalized.clone()) {
            return Err(ApiError::Request(eyre!(
                "public password group names must be unique"
            )));
        }
        let group = groups
            .get(&normalized)
            .wrap_request_err("public password group does not exist")?;
        if group.password_visibility != EnshroudedPasswordVisibility::Public {
            return Err(ApiError::Request(eyre!(
                "public password requires `password_visibility` to be `public`"
            )));
        }
        if group.role == EnshroudedServerRole::Admin || group.can_kick_ban {
            return Err(ApiError::Request(eyre!(
                "admin passwords cannot be public"
            )));
        }
    }

    let project_id = DBProjectId::from(project.inner.id).0;
    let mut transaction = pool
        .begin()
        .await
        .wrap_internal_err("starting server access transaction")?;
    sqlx::query!(
        "DELETE FROM enshrouded_server_public_passwords WHERE project_id = $1",
        project_id,
    )
    .execute(&mut transaction)
    .await
    .wrap_internal_err("clearing public enshrouded server passwords")?;
    for entry in &body.passwords {
        sqlx::query!(
            r#"
            INSERT INTO enshrouded_server_public_passwords (project_id, group_name, password)
            VALUES ($1, $2, $3)
            "#,
            project_id,
            entry.group_name.trim(),
            entry.password.trim(),
        )
		.execute(&mut transaction)
        .await
        .wrap_internal_err("saving public enshrouded server password")?;
    }
    transaction
        .commit()
        .await
        .wrap_internal_err("committing server access transaction")?;
    Ok(())
}
