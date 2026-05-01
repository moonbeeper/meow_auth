use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::user_session::UserSession as DbUserSession},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::types::{AlrightResponse, Session},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(current_session_info))
        .routes(routes!(list_sessions))
        .routes(routes!(delete_session))
        .routes(routes!(delete_all_sessions))
}

/// Get your current session info
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "current session info", body = Session),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn current_session_info(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Session>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

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
    responses(
        (status = 200, description = "a list of open sessions", body = Vec<Session>),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn list_sessions(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<Session>>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(sessions) = DbUserSession::find_many_by_user_id(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let sessions: Vec<_> = sessions.into_iter().map(Session::from).collect();
    Ok(Json(sessions))
}

#[derive(Debug, serde::Deserialize)]
pub struct SessionQuery {
    id: UlidId,
}

/// Close one of your sessions
///
/// You use the ID of an open session you have
#[utoipa::path(
    delete,
    path = "/{id}",
    params(
        ("id" = UlidId, description = "the id of the session to delete")
    ),
    responses(
        (status = 200, description = "successfully deleted the session"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_session(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(query): Path<SessionQuery>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(session)) = DbUserSession::find_by_pid(query.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    if session.user_id != auth.user_id() {
        return Err(ApiErrorCodes::SessionNotFound);
    }

    let mut tx = global.database.begin().await?;
    session.delete(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Close all your open sessions
///
/// Closes all your sessions, including your current one
#[utoipa::path(
    delete,
    path = "/all",
    responses(
        (status = 200, description = "successfully deleted all open sessions"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_all_sessions(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

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
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
