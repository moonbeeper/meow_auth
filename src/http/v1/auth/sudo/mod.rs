mod exchange;
mod start;

use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::sudo::{SudoOption, get_available_options},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{auth_manager::AuthContext, require_auth::RequireAuthenticationLayer},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(get_enable_options))
        .nest("/start", start::routes())
        .nest("/exchange", exchange::routes())
        .layer(RequireAuthenticationLayer::new())
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SudoOptionsResponse {
    options: Vec<SudoOption>,
}

/// Get the Sudo Re-Authentication options
///
/// Requests what authentications methods you can use to re-authenticate with the current user.
#[utoipa::path(
    get,
    path = "/",
    tags = ["sudo"],
    responses(
        (status = 200, description = "sudo re-authentication options", body = SudoOptionsResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn get_enable_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<SudoOptionsResponse>, ApiErrorCodes> {
    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let options = get_available_options(auth.user_id(), &global.database).await;

    Ok(Json(SudoOptionsResponse { options }))
}
