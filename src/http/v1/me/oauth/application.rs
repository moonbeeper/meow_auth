use std::{ops::Add, sync::Arc};

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{AuditAction, AuditEntry},
    auth::flags::UserFlag,
    database::{id::UlidId, models::oauth_application::OauthApplication as DbOauthApplication},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{
            auth_manager::AuthContext, browser_agent_manager::UserAgentContext,
            ip_manager::IpContext, require_user_flag::RequireUserFlagLayer,
        },
        v1::types::{AlrightResponse, ListDataRequest, ListDataResponse, OauthApplication},
        validator::Valid,
    },
    oauth::{
        scopes::{Scope, Scopes},
        secrets::get_secret_pair,
        valid_uri,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(create_application))
        .routes(routes!(update_application))
        .routes(routes!(delete_application))
        .routes(routes!(rotate_secret_application))
        .layer(RequireUserFlagLayer::new().forbid(UserFlag::CannotManageOauthApplications))
        .routes(routes!(list_applications))
        .routes(routes!(get_info_application))
}

/// List your oauth applications
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["oauth"],
    responses(
        (status = 200, description = "list of oauth applications", body = ListDataResponse<OauthApplication>),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn list_applications(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Query(request): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<OauthApplication>>, ApiErrorCodes> {
    let Ok(paginated) = DbOauthApplication::find_many_by_user_id_paginated(
        auth.user_id(),
        request.from,
        request.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let data: Vec<_> = paginated
        .items
        .into_iter()
        .map(OauthApplication::from)
        .collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct OauthApplicationData {
    #[validate(length(
        min = 4,
        max = 32,
        message = "name must be between 4 and 32 characters"
    ))]
    pub name: String,
    #[validate(url(message = "must be a valid url"))]
    pub redirect_uri: String,
    pub public: bool,
    pub scopes: i64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OauthApplicationDataResponse {
    pub id: UlidId,
    pub secret: String,
}

/// Create a new oauth application
#[utoipa::path(
    post,
    path = "/create",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully created oauth application", body = OauthApplicationDataResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn create_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Valid(Json(request)): Valid<Json<OauthApplicationData>>,
) -> Result<Json<OauthApplicationDataResponse>, ApiErrorCodes> {
    let mut scopes = Scopes::from_bits(request.scopes).sanitize(Scopes::all());
    if scopes.is_empty() {
        scopes = scopes.add(Scope::Profile);
    }

    let secret_pair = get_secret_pair(&global.settings);
    let redirect_uri = match url::Url::parse(&request.redirect_uri) {
        Ok(url) => url,
        Err(_) => return Err(ApiErrorCodes::InvalidRedirectUri),
    };

    if !valid_uri(&redirect_uri) {
        return Err(ApiErrorCodes::InvalidRedirectUri);
    }

    let app = DbOauthApplication::builder()
        .name(request.name)
        .redirect_uri(redirect_uri.to_string())
        .public(request.public)
        .scopes(scopes.bits())
        .secret(secret_pair.hash_bytes)
        .user_id(auth.user_id())
        .build();

    let mut tx = global.database.begin().await?;
    app.insert(&mut tx).await?;

    AuditEntry::builder()
        .user_id(auth.user_id())
        .actor_id(auth.user_id())
        .action(AuditAction::OauthApplicationCreated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: secret_pair.secret,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate, utoipa::IntoParams)]
pub struct OauthApplicationIdParam {
    pub id: UlidId,
}

/// Get info about a specific oauth application
#[utoipa::path(
    get,
    params(OauthApplicationIdParam),
    path = "/{id}",
    tags = ["oauth"],
    responses(
        (status = 200, description = "info about an oauth application", body = OauthApplication),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn get_info_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(request): Path<OauthApplicationIdParam>,
) -> Result<Json<OauthApplication>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthApplication::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    Ok(Json(app.into()))
}

/// Update an existing oauth application
#[utoipa::path(
    patch,
    params(OauthApplicationIdParam),
    path = "/{id}",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully updated the oauth application", body = AlrightResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn update_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<OauthApplicationIdParam>,
    Valid(Json(data)): Valid<Json<OauthApplicationData>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) = DbOauthApplication::find_by_id(request.id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let mut scopes = Scopes::from_bits(data.scopes).sanitize(Scopes::all());
    if scopes.is_empty() {
        scopes = scopes.add(Scope::Profile);
    }

    app.name = data.name;
    app.redirect_uri = data.redirect_uri;
    app.public = data.public;
    app.scopes = scopes.bits();

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;

    AuditEntry::builder()
        .user_id(auth.user_id())
        .actor_id(auth.user_id())
        .action(AuditAction::OauthApplicationUpdated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Delete an existing oauth application
#[utoipa::path(
    delete,
    params(OauthApplicationIdParam),
    path = "/{id}",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully deleted the oauth application"),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<OauthApplicationIdParam>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthApplication::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let mut tx = global.database.begin().await?;
    app.delete(&mut tx).await?;

    AuditEntry::builder()
        .user_id(auth.user_id())
        .actor_id(auth.user_id())
        .action(AuditAction::OauthApplicationDeleted)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Rotate the secret of an existing oauth application
#[utoipa::path(
    patch,
    params(OauthApplicationIdParam),
    path = "/{id}/rotate_keys",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully rotated the oauth application secret", body = OauthApplicationDataResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn rotate_secret_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<OauthApplicationIdParam>,
) -> Result<Json<OauthApplicationDataResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) = DbOauthApplication::find_by_id(request.id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let keys = get_secret_pair(&global.settings);
    app.secret = keys.hash_bytes;

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;

    AuditEntry::builder()
        .user_id(auth.user_id())
        .actor_id(auth.user_id())
        .action(AuditAction::OauthApplicationKeysRotated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: keys.secret,
    }))
}
