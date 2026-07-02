use std::sync::Arc;

use axum::extract::{Path, Query, State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::audit_log::AuditLog as DbAuditLog},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::types::{AuditLog, IdParam, ListDataRequest, ListDataResponse},
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
    Query(data): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<AuditLog>>, ApiErrorCodes> {
    let Ok(paginated) = DbAuditLog::find_many_by_user_id_with_logins_paginated(
        request.id,
        data.from,
        data.want_total.unwrap_or_default(),
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
