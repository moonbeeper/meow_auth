use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::oauth_application::OauthApplication as DbOauthApplication},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::{AlrightResponse, ListDataRequest, ListDataResponse, OauthApplication},
        validator::Valid,
    },
    oauth::{scopes::Scopes, secrets::get_secret_pair},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(list_applications))
        .routes(routes!(create_application))
        .routes(routes!(get_info_application))
        .routes(routes!(edit_application))
        .routes(routes!(delete_application))
        .routes(routes!(rotate_secret_application))
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
    Valid(Json(request)): Valid<Json<OauthApplicationData>>,
) -> Result<Json<OauthApplicationDataResponse>, ApiErrorCodes> {
    let scopes = Scopes::from_bits(request.scopes).sanitize(Scopes::all());
    let secret_pair = get_secret_pair(&global.settings);
    let app = DbOauthApplication::builder()
        .name(request.name)
        .redirect_uri(request.redirect_uri)
        .public(request.public)
        .scopes(scopes.bits())
        .secret(secret_pair.hash_bytes)
        .user_id(auth.user_id())
        .build();

    let mut tx = global.database.begin().await?;
    app.insert(&mut tx).await?;
    // audit::log(auth.user_id(), AuditAction::SessionDeleted, None, &mut tx).await?;
    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: secret_pair.secret,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct OauthApplicationIdParam {
    pub id: UlidId,
}

/// Get info about a specific oauth application
#[utoipa::path(
    get,
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
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    }

    Ok(Json(app.into()))
}

/// Update an existing oauth application
#[utoipa::path(
    patch,
    path = "/{id}",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully updated the oauth application", body = AlrightResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn edit_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(request): Path<OauthApplicationIdParam>,
    Valid(Json(data)): Valid<Json<OauthApplicationData>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) = DbOauthApplication::find_by_id(request.id, &global.database).await
    else {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    }

    let scopes = Scopes::from_bits(data.scopes).sanitize(Scopes::all());
    app.name = data.name;
    app.redirect_uri = data.redirect_uri;
    app.public = data.public;
    app.scopes = scopes.bits();

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Delete an existing oauth application
#[utoipa::path(
    delete,
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
    Path(request): Path<OauthApplicationIdParam>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthApplication::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    }

    let mut tx = global.database.begin().await?;
    app.delete(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Rotate the secret of an existing oauth application
#[utoipa::path(
    patch,
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
    Path(request): Path<OauthApplicationIdParam>,
) -> Result<Json<OauthApplicationDataResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) = DbOauthApplication::find_by_id(request.id, &global.database).await
    else {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::OauthApplicationNotFound);
    }

    let keys = get_secret_pair(&global.settings);
    app.secret = keys.hash_bytes;

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: keys.secret,
    }))
}
