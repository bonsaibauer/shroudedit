use std::time::Duration;
use xredis::RedisPool;

use actix_web::{HttpRequest, post, web};
use serde::{Deserialize, Serialize};

use crate::{
    auth::get_user_from_headers,
    database::PgPool,
    models::pats::Scopes,
    queue::{server_ping, session::AuthQueue},
    routes::ApiError,
    util::error::Context,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(ping_enshrouded);
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EnshroudedPingRequest {
    pub address: String,
    pub query_port: Option<u16>,
    pub timeout_ms: Option<u64>,
}

/// Ping an Enshrouded dedicated server through its UDP query port.
#[utoipa::path(
    context_path = "/server-ping",
    tag = "server ping",
    responses((status = OK, body = crate::models::exp::enshrouded::EnshroudedServerPingData))
)]
#[post("/enshrouded")]
pub async fn ping_enshrouded(
    req: HttpRequest,
    web::Json(request): web::Json<EnshroudedPingRequest>,
    pool: web::Data<PgPool>,
    redis: web::Data<RedisPool>,
    session_queue: web::Data<AuthQueue>,
) -> Result<
    web::Json<crate::models::exp::enshrouded::EnshroudedServerPingData>,
    ApiError,
> {
    let (_, _user) = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await
    .wrap_auth_err("authenticating API request")?;

    let query_port = request.query_port.unwrap_or(15637);
    if query_port == 0 {
        return Err(ApiError::Request(eyre::eyre!(
            "`query_port` must not be zero"
        )));
    }
    let timeout = request.timeout_ms.map(Duration::from_millis);
    let data = server_ping::ping_enshrouded_server(
        (request.address.as_str(), query_port),
        timeout,
    )
    .await
    .wrap_request_err("failed to ping Enshrouded server")?;

    Ok(web::Json(data))
}
