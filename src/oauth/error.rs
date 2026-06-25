use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use url::Url;

#[derive(Debug, serde::Serialize)]
pub struct OauthError {
    error: OauthErrorCodes,
    error_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    iss: String,
}

// should have custom error codes for application related stuff like... "hey, your id? no? get out" short circuit??
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthErrorCodes {
    InvalidClient,
    InvalidGrant,
    UnsupportedGrantType,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    InvalidRedirect,
    ServerError,
    TemporarilyUnavailble,
    // OIDC - skips account selection because currently you can only have one logged in account
    InteractionRequired,
    LoginRequired,
    ConsentRequired,
    // General Application
    InvalidRequest,
    InvalidToken,
    InsufficientScope,
}

impl OauthErrorCodes {
    pub fn as_str(&self) -> &'static str {
        match self {
            OauthErrorCodes::InvalidRequest => "invalid_request",
            OauthErrorCodes::InvalidClient => "invalid_client",
            OauthErrorCodes::InvalidGrant => "invalid_grant",
            OauthErrorCodes::UnsupportedGrantType => "unsupported_grant_type",
            OauthErrorCodes::UnauthorizedClient => "unauthorized_client",
            OauthErrorCodes::AccessDenied => "access_denied",
            OauthErrorCodes::UnsupportedResponseType => "unsupported_response_type",
            OauthErrorCodes::InvalidScope => "invalid_scope",
            OauthErrorCodes::InvalidRedirect => "invalid_redirect",
            OauthErrorCodes::ServerError => "server_error",
            OauthErrorCodes::TemporarilyUnavailble => "temporarily_unavailable",
            OauthErrorCodes::InteractionRequired => "interaction_required",
            OauthErrorCodes::LoginRequired => "login_required",
            OauthErrorCodes::ConsentRequired => "consent_required",
            OauthErrorCodes::InvalidToken => "invalid_token",
            OauthErrorCodes::InsufficientScope => "insufficient_scope",
        }
    }
    pub fn description(&self) -> &str {
        match self {
            _ => "meow meow meow meow meow wawa",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::IM_A_TEAPOT,
        }
    }
}

// TODO: return http 400 for token endpoint

impl OauthError {
    pub fn new(
        error: OauthErrorCodes,
        iss: &Url,
        custom_description: Option<String>,
        state: &Option<String>,
    ) -> Self {
        let description = match custom_description {
            Some(desc) => desc,
            None => error.description().to_string(),
        };
        OauthError {
            error_description: description,
            error,
            state: state.clone(),
            iss: iss.to_string(), // oncelock?
        }
    }
}

impl IntoResponse for OauthError {
    fn into_response(self) -> Response {
        (self.error.status_code(), axum::Json(self)).into_response()
    }
}
