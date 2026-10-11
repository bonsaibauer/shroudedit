use std::fmt::Write;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use crate::auth::get_user_from_headers;
use crate::auth::oauth::uris::{OAuthRedirectUris, ValidatedRedirectUri};
use crate::auth::validate::extract_authorization_header;
use crate::database::PgPool;
use crate::database::models::flow_item::DBFlow;
use crate::database::models::oauth_client_authorization_item::DBOAuthClientAuthorization;
use crate::database::models::oauth_client_item::DBOAuthClient;
use crate::database::models::oauth_token_item::DBOAuthAccessToken;
use crate::database::models::{
    DBOAuthClientAuthorizationId, generate_oauth_access_token_id,
    generate_oauth_client_authorization_id,
};
use crate::models;
use crate::models::ids::OAuthClientId;
use crate::models::pats::Scopes;
use crate::queue::session::AuthQueue;
use actix_web::http::header::{CACHE_CONTROL, LOCATION, PRAGMA};
use actix_web::web::{Data, Query};
use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::{DateTime, Duration};
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use xredis::RedisPool;

use self::errors::{OAuthError, OAuthErrorType};

use super::AuthenticationError;

pub mod errors;
pub mod uris;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(init_oauth)
        .service(accept_client_scopes)
        .service(reject_client_scopes)
        .service(request_token);
}

#[derive(Serialize, Deserialize)]
pub struct OAuthInit {
    pub client_id: OAuthClientId,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthClientAccessRequest {
    pub flow_id: String,
    pub client_id: OAuthClientId,
    pub client_name: String,
    pub client_icon: Option<String>,
    pub requested_scopes: Scopes,
}

#[utoipa::path(
	context_path = "/oauth",
	path = "/authorize",
	tag = "oauth",
	params(
		("client_id" = OAuthClientId, Query),
		("redirect_uri" = Option<String>, Query),
		("scope" = Option<String>, Query),
		("state" = Option<String>, Query),
		("code_challenge" = Option<String>, Query),
		("code_challenge_method" = Option<String>, Query)
	),
	responses((status = OK))
)]
#[get("authorize")]
pub async fn init_oauth(
    req: HttpRequest,
    Query(oauth_info): Query<OAuthInit>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    let user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::USER_AUTH_WRITE,
    )
    .await?
    .1;

    let client_id = oauth_info.client_id.into();
    let client = DBOAuthClient::get(client_id, &**pool).await?;

    if let Some(client) = client {
        let redirect_uri = ValidatedRedirectUri::validate(
            &oauth_info.redirect_uri,
            client.redirect_uris.iter().map(|r| r.uri.as_ref()),
            client.id,
        )?;
        let code_challenge = validate_code_challenge(
            oauth_info.code_challenge.as_deref(),
            oauth_info.code_challenge_method.as_deref(),
        )
        .map_err(|error| {
            OAuthError::redirect(error, &oauth_info.state, &redirect_uri)
        })?;

        let mut requested_scopes =
            oauth_info
                .scope
                .as_ref()
                .map_or(Ok(client.max_scopes), |s| {
                    Scopes::parse_from_oauth_scopes(s).map_err(|e| {
                        OAuthError::redirect(
                            OAuthErrorType::FailedScopeParse(e),
                            &oauth_info.state,
                            &redirect_uri,
                        )
                    })
                })?;

        if !crate::env::ENV.MONETIZATION_ENABLED {
            requested_scopes = requested_scopes
                - (Scopes::PAYOUTS_READ | Scopes::PAYOUTS_WRITE);
        }

        if !client.max_scopes.contains(requested_scopes) {
            return Err(OAuthError::redirect(
                OAuthErrorType::ScopesTooBroad,
                &oauth_info.state,
                &redirect_uri,
            ));
        }

        let existing_authorization =
            DBOAuthClientAuthorization::get(client.id, user.id.into(), &**pool)
                .await
                .map_err(|e| {
                    OAuthError::redirect(e, &oauth_info.state, &redirect_uri)
                })?;
        let redirect_uris = OAuthRedirectUris {
            original: oauth_info.redirect_uri.clone(),
            validated: redirect_uri.clone(),
        };
        match existing_authorization {
            Some(existing_authorization)
                if existing_authorization.scopes.contains(requested_scopes) =>
            {
                init_oauth_code_flow(
                    user.id.into(),
                    client.id.into(),
                    existing_authorization.id,
                    requested_scopes,
                    redirect_uris,
                    oauth_info.state,
                    code_challenge,
                    &redis,
                )
                .await
            }
            _ => {
                let flow_id = DBFlow::InitOAuthAppApproval {
                    user_id: user.id.into(),
                    client_id: client.id,
                    existing_authorization_id: existing_authorization
                        .map(|a| a.id),
                    scopes: requested_scopes,
                    redirect_uris,
                    state: oauth_info.state.clone(),
                    code_challenge,
                }
                .insert(Duration::minutes(30), &redis)
                .await
                .map_err(|e| {
                    OAuthError::redirect(e, &oauth_info.state, &redirect_uri)
                })?;

                let access_request = OAuthClientAccessRequest {
                    client_id: client.id.into(),
                    client_name: client.name,
                    client_icon: client.icon_url,
                    flow_id,
                    requested_scopes,
                };
                Ok(HttpResponse::Ok().json(access_request))
            }
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidClientId(
            client_id,
        )))
    }
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct RespondToOAuthClientScopes {
    pub flow: String,
}

#[utoipa::path(
	context_path = "/oauth",
	path = "/accept", tag = "oauth", responses((status = OK))
)]
#[post("accept")]
pub async fn accept_client_scopes(
    req: HttpRequest,
    accept_body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    accept_or_reject_client_scopes(
        true,
        req,
        accept_body,
        pool,
        redis,
        session_queue,
    )
    .await
}

#[utoipa::path(
	context_path = "/oauth",
	path = "/reject", tag = "oauth", responses((status = OK))
)]
#[post("reject")]
pub async fn reject_client_scopes(
    req: HttpRequest,
    body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    accept_or_reject_client_scopes(false, req, body, pool, redis, session_queue)
        .await
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: Option<String>,
    pub client_id: models::ids::OAuthClientId,
    pub code_verifier: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[utoipa::path(
	context_path = "/oauth",
	path = "/token", tag = "oauth", responses((status = OK))
)]
#[post("token")]
/// Params should be in the urlencoded request body
/// Confidential clients provide their secret in the authorization header. Public clients use PKCE.
/// Per IETF RFC6749 Section 4.1.3 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.3)
pub async fn request_token(
    req: HttpRequest,
    req_params: web::Form<TokenRequest>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
) -> Result<HttpResponse, OAuthError> {
    let req_client_id = req_params.client_id;
    let client = DBOAuthClient::get(req_client_id.into(), &**pool).await?;
    if let Some(client) = client {
        if req_params.grant_type != "authorization_code" {
            return Err(OAuthError::error(
                OAuthErrorType::OnlySupportsAuthorizationCodeGrant(
                    req_params.grant_type.clone(),
                ),
            ));
        }

        let candidate = DBFlow::get(&req_params.code, &redis).await?;
        let Some(DBFlow::OAuthAuthorizationCodeSupplied {
            client_id: flow_client_id,
            original_redirect_uri,
            code_challenge,
            ..
        }) = candidate.as_ref()
        else {
            return Err(OAuthError::error(OAuthErrorType::InvalidAuthCode));
        };

        if req_client_id != (*flow_client_id).into() {
            return Err(OAuthError::error(OAuthErrorType::UnauthorizedClient));
        }

        if original_redirect_uri != &req_params.redirect_uri {
            return Err(OAuthError::error(OAuthErrorType::RedirectUriChanged(
                req_params.redirect_uri.clone(),
            )));
        }

        if let Some(challenge) = code_challenge {
            verify_code_verifier(challenge, req_params.code_verifier.as_deref())?;
        } else {
            authenticate_client_token_request(&req, &client)?;
        }

        // Validate the proof before consuming the one-use authorization code.
        let flow = DBFlow::take_if(
            &req_params.code,
            |flow| {
                matches!(
                    flow,
                    DBFlow::OAuthAuthorizationCodeSupplied {
                        client_id,
                        code_challenge: stored_challenge,
                        ..
                    } if client_id == flow_client_id && stored_challenge == code_challenge
                )
            },
            &redis,
        )
        .await?;
        if let Some(DBFlow::OAuthAuthorizationCodeSupplied {
            user_id,
            client_id,
            authorization_id,
            scopes,
            original_redirect_uri,
            code_challenge: _,
        }) = flow
        {
            let scopes = scopes - Scopes::restricted();

            let mut transaction = pool.begin().await?;
            let token_id =
                generate_oauth_access_token_id(&mut transaction).await?;
            let token = generate_access_token();
            let token_hash = DBOAuthAccessToken::hash_token(&token);
            let time_until_expiration = DBOAuthAccessToken {
                id: token_id,
                authorization_id,
                token_hash,
                scopes,
                created: DateTime::default(),
                expires: DateTime::default(),
                last_used: None,
                client_id,
                user_id,
            }
            .insert(&mut transaction)
            .await?;

            transaction.commit().await?;

            // IETF RFC6749 Section 5.1 (https://datatracker.ietf.org/doc/html/rfc6749#section-5.1)
            Ok(HttpResponse::Ok()
                .append_header((CACHE_CONTROL, "no-store"))
                .append_header((PRAGMA, "no-cache"))
                .json(TokenResponse {
                    access_token: token,
                    token_type: "Bearer".to_string(),
                    expires_in: time_until_expiration.num_seconds(),
                }))
        } else {
            Err(OAuthError::error(OAuthErrorType::InvalidAuthCode))
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidClientId(
            req_client_id.into(),
        )))
    }
}

pub async fn accept_or_reject_client_scopes(
    accept: bool,
    req: HttpRequest,
    body: web::Json<RespondToOAuthClientScopes>,
    pool: Data<PgPool>,
    redis: Data<RedisPool>,
    session_queue: Data<AuthQueue>,
) -> Result<HttpResponse, OAuthError> {
    let current_user = get_user_from_headers(
        &req,
        &**pool,
        &redis,
        &session_queue,
        Scopes::SESSION_ACCESS,
    )
    .await?
    .1;

    let flow = DBFlow::take_if(
        &body.flow,
        |f| matches!(f, DBFlow::InitOAuthAppApproval { .. }),
        &redis,
    )
    .await?;
    if let Some(DBFlow::InitOAuthAppApproval {
        user_id,
        client_id,
        existing_authorization_id,
        scopes,
        redirect_uris,
        state,
        code_challenge,
    }) = flow
    {
        if current_user.id != user_id.into() {
            return Err(OAuthError::error(
                AuthenticationError::InvalidCredentials,
            ));
        }

        if accept {
            let mut transaction = pool.begin().await?;

            let auth_id = match existing_authorization_id {
                Some(id) => id,
                None => {
                    generate_oauth_client_authorization_id(&mut transaction)
                        .await?
                }
            };
            DBOAuthClientAuthorization::upsert(
                auth_id,
                client_id,
                user_id,
                scopes,
                &mut transaction,
            )
            .await?;

            transaction.commit().await?;

            init_oauth_code_flow(
                user_id,
                client_id.into(),
                auth_id,
                scopes,
                redirect_uris,
                state,
                code_challenge,
                &redis,
            )
            .await
        } else {
            Err(OAuthError::redirect(
                OAuthErrorType::AccessDenied,
                &state,
                &redirect_uris.validated,
            ))
        }
    } else {
        Err(OAuthError::error(OAuthErrorType::InvalidAcceptFlowId))
    }
}

fn authenticate_client_token_request(
    req: &HttpRequest,
    client: &DBOAuthClient,
) -> Result<(), OAuthError> {
    let client_secret = extract_authorization_header(req)?;
    let hashed_client_secret = DBOAuthClient::hash_secret(client_secret);
    if client.secret_hash != hashed_client_secret {
        Err(OAuthError::error(
            OAuthErrorType::ClientAuthenticationFailed,
        ))
    } else {
        Ok(())
    }
}

fn is_pkce_value(value: &str) -> bool {
	(43..=128).contains(&value.len())
		&& value
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || b"-._~".contains(&byte))
}

fn validate_code_challenge(
	challenge: Option<&str>,
	method: Option<&str>,
) -> Result<Option<String>, OAuthErrorType> {
	match (challenge, method) {
		(None, None) => Ok(None),
		(Some(challenge), Some("S256"))
			if is_pkce_value(challenge) && challenge.len() == 43 =>
		{
			Ok(Some(challenge.to_owned()))
		}
		_ => Err(OAuthErrorType::InvalidPkceParameters),
	}
}

fn verify_code_verifier(
	challenge: &str,
	verifier: Option<&str>,
) -> Result<(), OAuthError> {
	let verifier = verifier
		.filter(|verifier| is_pkce_value(verifier))
		.ok_or_else(|| OAuthError::error(OAuthErrorType::InvalidAuthCode))?;
	let challenge_from_verifier = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
	if challenge_from_verifier != challenge {
		return Err(OAuthError::error(OAuthErrorType::InvalidAuthCode));
	}
	Ok(())
}

fn generate_access_token() -> String {
    let random = ChaCha20Rng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(60)
        .map(char::from)
        .collect::<String>();
    format!("mro_{random}")
}

async fn init_oauth_code_flow(
    user_id: crate::database::models::DBUserId,
    client_id: OAuthClientId,
    authorization_id: DBOAuthClientAuthorizationId,
    scopes: Scopes,
    redirect_uris: OAuthRedirectUris,
    state: Option<String>,
    code_challenge: Option<String>,
    redis: &RedisPool,
) -> Result<HttpResponse, OAuthError> {
    let code = DBFlow::OAuthAuthorizationCodeSupplied {
        user_id,
        client_id: client_id.into(),
        authorization_id,
        scopes,
        original_redirect_uri: redirect_uris.original.clone(),
        code_challenge,
    }
    .insert(Duration::minutes(10), redis)
    .await
    .map_err(|e| {
        OAuthError::redirect(e, &state, &redirect_uris.validated.clone())
    })?;

    let mut redirect_params = vec![format!("code={code}")];
    if let Some(state) = state {
        redirect_params.push(format!("state={state}"));
    }

    let redirect_uri =
        append_params_to_uri(&redirect_uris.validated.0, &redirect_params);

    // IETF RFC 6749 Section 4.1.2 (https://datatracker.ietf.org/doc/html/rfc6749#section-4.1.2)
    Ok(HttpResponse::Ok()
        .append_header((LOCATION, redirect_uri.clone()))
        .body(redirect_uri))
}

fn append_params_to_uri(uri: &str, params: &[impl AsRef<str>]) -> String {
    let mut uri = uri.to_string();
    let mut connector = if uri.contains('?') { "&" } else { "?" };
    for param in params {
        write!(&mut uri, "{connector}{}", param.as_ref()).unwrap();
        connector = "&";
    }

    uri
}
