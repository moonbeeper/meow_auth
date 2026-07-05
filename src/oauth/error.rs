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
    // UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    InvalidRedirect,
    ServerError,
    // TemporarilyUnavailble,
    // OIDC - skips account selection because currently you can only have one logged in account
    // InteractionRequired,
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
            // OauthErrorCodes::UnauthorizedClient => "unauthorized_client",
            OauthErrorCodes::AccessDenied => "access_denied",
            OauthErrorCodes::UnsupportedResponseType => "unsupported_response_type",
            OauthErrorCodes::InvalidScope => "invalid_scope",
            OauthErrorCodes::InvalidRedirect => "invalid_redirect",
            OauthErrorCodes::ServerError => "server_error",
            // OauthErrorCodes::TemporarilyUnavailble => "temporarily_unavailable",
            // OauthErrorCodes::InteractionRequired => "interaction_required",
            OauthErrorCodes::LoginRequired => "login_required",
            OauthErrorCodes::ConsentRequired => "consent_required",
            OauthErrorCodes::InvalidToken => "invalid_token",
            OauthErrorCodes::InsufficientScope => "insufficient_scope",
        }
    }
    pub fn description(&self) -> &str {
        match self {
            OauthErrorCodes::InvalidClient => "client authentication failed (e.g. unknown client)",
            OauthErrorCodes::InvalidGrant => {
                "the authorization grant (e.g. authorization code) is invalid, expired, has already been used, or was issued to another client"
            }
            OauthErrorCodes::UnsupportedGrantType => {
                "only the authorization_code grant type is supported"
            }
            // OauthErrorCodes::UnauthorizedClient => todo!(),
            OauthErrorCodes::AccessDenied => {
                "the request was denied (e.g. user denied access or not authenticated)"
            }
            OauthErrorCodes::UnsupportedResponseType => "only the code response type is supported",
            OauthErrorCodes::InvalidScope => {
                "the requested scope exceeds the scopes allowed for this client"
            }
            OauthErrorCodes::InvalidRedirect => {
                "the provided redirect_uri does not match the client's registered redirect_uri"
            }
            OauthErrorCodes::ServerError => {
                "an unexpected server error occurred. this made the server go boom. please try again later"
            }
            // OauthErrorCodes::TemporarilyUnavailble => "the server is temporarily unable to handle the request",
            // OauthErrorCodes::InteractionRequired => {
            //     "user interaction is required to complete the request"
            // }
            OauthErrorCodes::LoginRequired => {
                "an authenticated session is required to complete the request"
            }
            OauthErrorCodes::ConsentRequired => "user consent is required to complete the request",
            OauthErrorCodes::InvalidRequest => {
                "the request is missing a required parameter or contains an invalid value (e.g. redirect_uri, code_verifier)"
            }
            OauthErrorCodes::InvalidToken => {
                "the access token is invalid, expired, revoked, or the associated user no longer exists (went to the void realm)"
            }
            OauthErrorCodes::InsufficientScope => {
                "the access token does not have the required scopes to access the requested resource"
            }
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            OauthErrorCodes::InvalidClient => StatusCode::UNAUTHORIZED,
            OauthErrorCodes::InvalidGrant => StatusCode::BAD_REQUEST,
            OauthErrorCodes::UnsupportedGrantType => StatusCode::BAD_REQUEST,
            OauthErrorCodes::AccessDenied => StatusCode::FORBIDDEN,
            OauthErrorCodes::UnsupportedResponseType => StatusCode::BAD_REQUEST,
            OauthErrorCodes::InvalidScope => StatusCode::BAD_REQUEST,
            OauthErrorCodes::InvalidRedirect => StatusCode::BAD_REQUEST,
            OauthErrorCodes::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            OauthErrorCodes::LoginRequired => StatusCode::BAD_REQUEST,
            OauthErrorCodes::ConsentRequired => StatusCode::BAD_REQUEST,
            OauthErrorCodes::InvalidRequest => StatusCode::BAD_REQUEST,
            OauthErrorCodes::InvalidToken => StatusCode::UNAUTHORIZED,
            OauthErrorCodes::InsufficientScope => StatusCode::FORBIDDEN,
        }
    }
}

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
