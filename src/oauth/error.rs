use axum::{
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use url::Url;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthErrorCodes {
    InvalidRequest,
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
}

impl OauthErrorCodes {
    fn as_str(&self) -> &'static str {
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
        }
    }

    fn description(&self) -> &str {
        "todo" // TODO: descriptions for oauth error codes
    }
}

// TODO: return http 400 for token endpoint

#[derive(Debug, serde::Serialize)]
pub struct OauthError {
    error: OauthErrorCodes,
    error_description: String,
    state: Option<String>,
    iss: String,
}

impl OauthError {
    pub fn new(error: OauthErrorCodes, iss: &Url, state: &Option<String>) -> Self {
        OauthError {
            error_description: error.description().to_string(),
            error,
            state: state.clone(),
            iss: iss.to_string(), // oncelock?
        }
    }

    pub fn description(mut self, description: String) -> Self {
        self.error_description = description;
        self
    }
}

impl IntoResponse for OauthError {
    fn into_response(self) -> Response {
        // TODO: GREAT HARDCODED GREEEAT
        (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(self)).into_response()
    }
}

pub fn redirect_to(url: &Url) -> Response {
    Redirect::to(url.as_str()).into_response()
}

pub fn redirect_with_error(mut url: Url, error: OauthError) -> Response {
    // bruh
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("error", error.error.as_str());
        q.append_pair("error_description", &error.error_description);
        q.append_pair("iss", &error.iss);

        if let Some(state) = error.state {
            q.append_pair("state", &state);
        }
    }

    Redirect::to(url.as_str()).into_response()
}
