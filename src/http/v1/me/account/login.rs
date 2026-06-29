use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    auth::{RE_AUTH_FLOW_LOGIN, flags::UserFlag, mailer::AuthMailer},
    database::models::user::User,
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{auth_manager::AuthContext, require_user_flag::RequireUserFlagLayer},
        v1::types::AlrightResponse,
        validator::Valid,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(change_user_login))
        .layer(RequireUserFlagLayer::new().forbid(UserFlag::CannotModifyLogin))
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ChangeLoginRequest {
    #[validate( // mr fmt doesnt format this aberration.
        length(min = 3, max = 63, message = "must be between 4 letters and 64"), // counts from 0 duh
        regex(path = *RE_AUTH_FLOW_LOGIN, message = "must be alphanumeric and can contain underscores")
    )]
    login: String,
}

/// Change your current login (username)
#[utoipa::path(
    patch,
    path = "/login",
    tags = ["user"],
    responses(
        (status = 200, description = "successfully created the change request", body = AlrightResponse),
        (status = 401, description = "sudo not enabled", body = ApiError),
        (status = 400, description = "new login is already associated", body = ApiError),
        (status = 403, description = "tried to change login too soon after changing it", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn change_user_login(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Valid(Json(request)): Valid<Json<ChangeLoginRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let Ok(None) = User::find_by_login(request.login.clone(), &global.database).await else {
        return Err(ApiErrorCodes::LoginAlreadyAssociated);
    };

    if user.login_updated_at + chrono::Duration::days(24) > chrono::Utc::now() {
        return Err(ApiErrorCodes::LoginChangeTooSoon);
    }

    user.login = request.login.clone();
    user.login_updated_at = chrono::Utc::now();

    let mut tx = global.database.begin().await?;
    user.update(&mut tx).await?;
    audit::log(auth.user_id(), AuditAction::LoginChanged, None, &mut tx).await?;
    tx.commit().await?;

    AuthMailer::login_updated(user.login, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}
