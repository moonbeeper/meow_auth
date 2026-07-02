mod email;
mod login;

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::audit_log::AuditLog as DbAuditLog,
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::{AuditLog, ListDataRequest, ListDataResponse},
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
    Query(request): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<AuditLog>>, ApiErrorCodes> {
    let Ok(paginated) = DbAuditLog::find_many_by_user_id_paginated(
        auth.user_id(),
        request.from,
        request.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let data: Vec<_> = paginated.items.into_iter().map(AuditLog::from).collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}
