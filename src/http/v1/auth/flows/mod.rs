mod exchange;
mod info;
mod start;

use std::sync::Arc;

use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::user::User},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::require_auth::RequireAuthenticationLayer,
        v1::types::AuthMethod,
    },
};

// TODO: should use correctly errors.
// TODO: should have validation of these things
// TODO: should let the user log in via their username, maybe by using a regex to determine if the input is an email or login

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(flow_options))
        .nest("/start", start::routes())
        .nest("/exchange", exchange::routes())
        .nest("/info", info::routes())
        .layer(RequireAuthenticationLayer::new().need_auth(false))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct FlowRequest {
    login: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowResponse {
    pub flow_id: UlidId,
    pub next_method: Vec<AuthMethod>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowOptionResponse {
    pub methods: Vec<AuthMethod>,
}

/// Get the Authentication options
///
/// Requests what authentications methods you can use to authenticate with the given login.
/// This is useful for determining if a user has a passkey or not
#[utoipa::path(
    post,
    path = "/",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication options", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn flow_options(
    State(global): State<Arc<GlobalState>>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<FlowOptionResponse>, ApiErrorCodes> {
    let mut methods = vec![AuthMethod::Otp];

    let Ok(Some(user)) = User::find_by_login(request.login, &global.database).await else {
        return Ok(Json(FlowOptionResponse { methods }));
    };

    if user.has_webauthn {
        methods.push(AuthMethod::Passkey)
    }
    Ok(Json(FlowOptionResponse { methods }))
}
