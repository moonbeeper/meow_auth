mod exchange;
mod info;
mod start;

use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::user::User},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::types::AuthMethod,
    },
};

// TODO: should use correctly errors.
// TODO: should have validation of these things
// TODO: should let the user log in via their username, maybe by using a regex to determine if the input is an email or login

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(options))
        .nest("/start", start::routes())
        .nest("/exchange", exchange::routes())
        .nest("/info", info::routes())
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct FlowRequest {
    email: String, // should also allow login via regex stuff
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

/// Get the login options
///
/// This returns what authentication method you can use to login. It will be used as the login/start
#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "login options", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<FlowOptionResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let mut methods = vec![];
    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Ok(Json(FlowOptionResponse { methods }));
    };

    methods.push(AuthMethod::Otp);
    if user.has_webauthn {
        methods.push(AuthMethod::Passkey)
    }
    Ok(Json(FlowOptionResponse { methods }))
}
