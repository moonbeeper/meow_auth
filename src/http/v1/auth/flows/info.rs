use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::user::User,
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(login_available))
        .routes(routes!(email_available))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema, Default)]
pub struct InfoAvailableResponse {
    available: bool,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct InfoAvailableRequest {
    who: String,
}

/// Fetch if a login is available for registration
#[utoipa::path(
    post,
    path = "/login",
    responses(
        (status = 200, description = "result of availability", body = InfoAvailableResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn login_available(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<InfoAvailableRequest>,
) -> Result<Json<InfoAvailableResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(None) = User::find_by_login(request.who, &global.database).await else {
        return Ok(Json(InfoAvailableResponse::default()));
    };

    Ok(Json(InfoAvailableResponse { available: true }))
}

/// Fetch if an email is available for registration
#[utoipa::path(
    post,
    path = "/email",
    responses(
        (status = 200, description = "result of availability", body = InfoAvailableResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn email_available(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<InfoAvailableRequest>,
) -> Result<Json<InfoAvailableResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(None) = User::find_by_email(request.who, &global.database).await else {
        return Ok(Json(InfoAvailableResponse::default()));
    };

    Ok(Json(InfoAvailableResponse { available: true }))
}
