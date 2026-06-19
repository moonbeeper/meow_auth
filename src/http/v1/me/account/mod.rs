mod email;
mod login;

use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::audit_log::AuditLog as DbAuditLog,
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::AuditLog,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .merge(email::routes())
        .merge(login::routes())
        .routes(routes!(current_user_audit_log))
}

/// Get your current user audit log
#[utoipa::path(
    get,
    path = "/audit",
    tags = ["user"],
    responses(
        (status = 200, description = "current user audit log", body = AuditLog),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn current_user_audit_log(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<AuditLog>>, ApiErrorCodes> {
    let Ok(audit_log) = DbAuditLog::find_by_user(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let log: Vec<_> = audit_log.into_iter().map(AuditLog::from).collect();
    Ok(Json(log))
}
