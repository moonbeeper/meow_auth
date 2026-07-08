use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    auth::flags::UserFlag,
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
        .routes(routes!(change_user_name))
        .layer(RequireUserFlagLayer::new().forbid(UserFlag::CannotModifyName))
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ChangeNameRequest {
    #[validate( // mr fmt doesnt format this aberration.
        length(min = 3, max = 50, message = "must be between 4 letters and 64"), // counts from 0 duh
        // regex(path = *RE_AUTH_FLOW_LOGIN, message = "must be alphanumeric and can contain underscores")
    )]
    name: String,
}

/// Change your current name
#[utoipa::path(
    patch,
    path = "/login",
    tags = ["user"],
    responses(
        (status = 200, description = "successfully changed the name", body = AlrightResponse),
        // (status = 401, description = "sudo not enabled", body = ApiError),
        // (status = 400, description = "new login is already associated", body = ApiError),
        (status = 403, description = "tried to change name too soon after changing it", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn change_user_name(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Valid(Json(request)): Valid<Json<ChangeNameRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if user.name_updated_at + chrono::Duration::days(24) > chrono::Utc::now() {
        return Err(ApiErrorCodes::NameChangeTooSoon);
    }

    let mut flags = auth.user_flags();

    if !flags.has(UserFlag::HasSetName) {
        flags = flags.add(UserFlag::HasSetName)
    }

    user.name = request.name.clone();
    user.name_updated_at = chrono::Utc::now();
    user.flags = flags.bits();

    let mut tx = global.database.begin().await?;
    user.update(&mut tx).await?;
    audit::log(
        auth.user_id(),
        auth.user_id(),
        AuditAction::NameChanged,
        None,
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
