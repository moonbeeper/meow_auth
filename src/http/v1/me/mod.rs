mod account;
mod oauth;

use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    database::models::{user::User as DbUser, user_session::UserSession},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{auth_manager::AuthContext, require_auth::RequireAuthenticationLayer},
        v1::types::{AlrightResponse, User},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(current_user_info))
        .routes(routes!(logout))
        .nest("/account", account::routes())
        .nest("/oauth", oauth::routes())
        .layer(RequireAuthenticationLayer::new())
}

/// Get your current user information
#[utoipa::path(
    get,
    path = "/",
    tags = ["user"],
    responses(
        (status = 200, description = "current user info", body = User),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn current_user_info(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<User>, ApiErrorCodes> {
    let Ok(Some(user)) = DbUser::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let user = User::from(user);
    Ok(Json(user))
}

/// Log out of this session
#[utoipa::path(
    post,
    path = "/logout",
    tags = ["user"],
    responses(
        (status = 200, description = "successfully logged out"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn logout(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let mut tx = global.database.begin().await?;
    UserSession::delete_by_id(auth.session_id(), &mut tx).await?;
    audit::log(auth.user_id(), AuditAction::SessionDeleted, None, &mut tx).await?;
    tx.commit().await?;
    Ok(Json(AlrightResponse::default()))
}
