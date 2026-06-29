use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    database::{id::UlidId, models::user_session::UserSession as DbUserSession},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::{AlrightResponse, ListDataRequest, ListDataResponse, Session},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(current_session_info))
        .routes(routes!(list_sessions))
        .routes(routes!(delete_session))
        .routes(routes!(delete_all_sessions))
}

/// Current session info
#[utoipa::path(
    get,
    path = "/",
    tags = ["sessions"],
    responses(
        (status = 200, description = "current session info", body = Session),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn current_session_info(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Session>, ApiErrorCodes> {
    let Ok(Some(session)) = DbUserSession::find_by_id(auth.session_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let session = Session::from(session);
    Ok(Json(session))
}

/// List all your open sessions
#[utoipa::path(
    get,
    path = "/list",
    tags = ["sessions"],
    responses(
        (status = 200, description = "list of open sessions", body = ListDataResponse<Session>),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn list_sessions(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Query(request): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<Session>>, ApiErrorCodes> {
    let Ok(paginated) = DbUserSession::find_many_by_user_id_paginated(
        auth.user_id(),
        request.from,
        request.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let data: Vec<_> = paginated.items.into_iter().map(Session::from).collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct SessionQuery {
    id: UlidId,
}

/// Close one of your sessions
#[utoipa::path(
    delete,
    path = "/{id}",
    tags = ["sessions"],
    params(
        ("id" = UlidId, description = "the id of the session to delete")
    ),
    responses(
        (status = 200, description = "successfully revoked the session"),
        (status = 401, description = "sudo not enabled", body = ApiError),
        (status = 404, description = "session not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_session(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(query): Path<SessionQuery>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(session)) = DbUserSession::find_by_pid(query.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    if session.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("session"));
    }

    let mut tx = global.database.begin().await?;
    session.delete(&mut tx).await?;
    audit::log(auth.user_id(), AuditAction::SessionDeleted, None, &mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Close all your open sessions
///
/// This includes your current one
#[utoipa::path(
    delete,
    path = "/all",
    tags = ["sessions"],
    responses(
        (status = 200, description = "successfully revoked all open sessions"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_all_sessions(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(sessions) = DbUserSession::find_many_by_user_id(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let ids = sessions.into_iter().map(|v| v.id).collect();

    let mut tx = global.database.begin().await?;
    DbUserSession::delete_many_by_id(ids, &mut tx).await?;
    audit::log(auth.user_id(), AuditAction::SessionsDeleted, None, &mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
