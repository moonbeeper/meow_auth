mod exchange;
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
        middleware::{ratelimit_manager::RatelimitLayer, require_auth::RequireAuthenticationLayer},
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
        .layer(RequireAuthenticationLayer::new().need_auth(false))
        .layer(RatelimitLayer::new(20, chrono::Duration::seconds(60)))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct FlowRequest {
    /// The email address of the user that is trying to authenticate
    email: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowResponse {
    /// The flow ID used to identify and track this authentication flow
    pub flow_id: UlidId,
    /// The next authentication method that can be used to finalize the authentication flow
    pub next_method: Vec<AuthMethod>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowOptionResponse {
    /// The available authentication methods that an user can use to authenticate
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

    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Ok(Json(FlowOptionResponse { methods }));
    };

    if user.has_webauthn {
        methods.push(AuthMethod::Passkey)
    }
    Ok(Json(FlowOptionResponse { methods }))
}
