use std::sync::Arc;

use axum::extract::{Path, Query, State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::user_session::UserSession as DbUserSession},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::types::{
            AlrightResponse, IdParam, ListDataRequest, ListDataResponse, Session, TwoIdParam,
        },
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(admin_list_user_sessions))
        .routes(routes!(admin_revoke_all_user_sessions))
        .routes(routes!(admin_revoke_user_session))
}

/// List sessions of a user
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "list of user sessions", body = ListDataResponse<Session>),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_list_user_sessions(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
    Query(data): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<Session>>, ApiErrorCodes> {
    let Ok(paginated) = DbUserSession::find_many_by_user_id_paginated(
        request.id,
        data.from,
        data.want_total.unwrap_or_default(),
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

/// Revoke a user session
#[utoipa::path(
    delete,
    path = "/{cid}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the user session")
    ),
    responses(
        (status = 200, description = "successfully revoked the user session", body = AlrightResponse),
        (status = 404, description = "user session not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_revoke_user_session(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(session)) = DbUserSession::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("session"));
    };

    if session.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("session"));
    }

    let mut tx = global.database.begin().await?;
    session.delete(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Revoke all user sessions
#[utoipa::path(
    delete,
    path = "/",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "successfully revoked all user sessions", body = AlrightResponse),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    ))]
pub async fn admin_revoke_all_user_sessions(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let mut tx = global.database.begin().await?;
    DbUserSession::delete_all_by_user_id(request.id, &mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
