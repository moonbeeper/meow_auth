use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::user::User as DbUser,
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::types::User,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(current_user_info))
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
