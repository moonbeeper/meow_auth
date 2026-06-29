use std::sync::Arc;

use axum::extract::{Path, State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::audit_log::AuditLog as DbAuditLog},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::types::{AuditLog, IdParam},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(admin_user_audit_log))
}

/// Get a user's audit log
#[utoipa::path(
    get,
    path = "/",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "current user audit log", body = AuditLog),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_user_audit_log(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
) -> Result<Json<Vec<AuditLog>>, ApiErrorCodes> {
    let Ok(audit_log) = DbAuditLog::find_by_user(request.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let log: Vec<_> = audit_log.into_iter().map(AuditLog::from).collect();
    Ok(Json(log))
}
