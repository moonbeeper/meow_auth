use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::{user::User as DbUser, user_session::UserSession},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::types::User,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(current_user_info))
        .routes(routes!(logout))
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "current user info", body = User),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn current_user_info(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<User>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(user)) = DbUser::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let user = User::from(user);
    Ok(Json(user))
}

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 200, description = "successfully logged out"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn logout(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<(), ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(session)) = UserSession::find_by_id(auth.session_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let mut tx = global.database.begin().await?;
    session.delete(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}
