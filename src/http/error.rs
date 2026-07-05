use axum::{http::StatusCode, response::IntoResponse};

use crate::database::error::DatabaseError;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiError<'a> {
    ok: bool,
    code: &'a str,
    message: String,
}

// TODO: Should merge all common errors. To not have a thousand "not found" errors.

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
pub enum ApiErrorCodes {
    #[error("meow meow")]
    OtpExpired,
    #[error("you are already authenticated")]
    AlreadyAuthenticated,
    #[error("you are unauthenticated")]
    Unauthenticated,
    #[error("meow *blep*")]
    Meow,
    #[error("an error occurred while interacting with the database")]
    Database(#[from] DatabaseError),
    #[error("an error occurred while interacting with the database")]
    RouteDatabase(#[from] sqlx::Error),
    #[error("an unknown error occurred")]
    InternalServerError,
    #[error("totp is already enabled for this account")]
    TotpAlreadyEnabled,
    #[error("the totp flow was not started before exchanging")]
    TotpFlowNotFound,
    #[error("totp is not enabled for this account")]
    TotpNotEnabled,
    #[error("totp recovery code is already used")]
    TotpRecoveryAlreadyUsed,
    #[error("sudo is not enabled for this session")]
    SudoNotEnabled,
    #[error("sudo is already enabled for this session")]
    SudoAlreadyEnabled,
    #[error("chosen sudo option is not available")]
    SudoOptionNotAvailable,
    #[error("the provided login is invalid")]
    AccountNotFound, // man. this hurts a bit.
    #[error("the provided email is already associated with another account")]
    EmailAlreadyAssociated, // man. this hurts a bit.
    #[error("the provided login is already associated with another account")]
    LoginAlreadyAssociated, // man. this hurts a bit.
    #[error("the provided code is invalid")]
    InvalidCode,
    #[error("webauthn is not enabled for this account")]
    WebauthnNotEnabled,
    #[error("you need totp enabled to make this action")]
    TotpRequiredEnabled,
    #[error("the requested webauthn challenge wasn't found")]
    WebauthnChallengeNotFound,
    // idk what to put in this error message. like, service??
    #[error("an error occurred while interacting with the webauthn service")]
    WebauthnError(#[from] webauthn_rs::prelude::WebauthnError),
    #[error("deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("this account's passkey was likely compromised. New attempts will be blocked")]
    WebauthnCompromised,
    #[error("an error occurred while sending an email")]
    MailerError(#[from] crate::mailer::error::MailerErrors),
    /// ->
    #[error("Failed to deserialize the body into the target type: {0}")]
    DataError(String),
    #[error("Failed to parse the request body as JSON: {0}")]
    JsonSyntaxError(String),
    #[error("Expected request with `Content-Type: application/json`")]
    MissingJsonContentType,
    #[error("Failed to buffer the request body")]
    FailedToBufferContent,
    /// ->
    #[error("{0}")]
    ValidationError(String),
    /// ->
    #[error("this action is already verified")]
    AlreadyVerified,
    #[error("this action's flow was not found")]
    FlowNotFound,
    #[error("you cannot change your login so soon after the last change")]
    LoginChangeTooSoon,
    // lord moon has ripped a feather from my wings to make your desires true.
    // Merged some "*NotFound" errors into this one.
    #[error("the requested {0} was not found")]
    DataNotFound(&'static str),
    #[error("the requested action is not allowed for this account")]
    ActionBlocked,
    #[error("you have reached the rate limit for this action. try again later.")]
    RatelimitExceeded,
}

// wtf
impl From<()> for ApiErrorCodes {
    fn from(_: ()) -> Self {
        ApiErrorCodes::Meow
    }
}

// TODO: Should trace the error.
impl From<anyhow::Error> for ApiErrorCodes {
    fn from(_: anyhow::Error) -> Self {
        Self::InternalServerError
    }
}

impl ApiErrorCodes {
    fn as_api_error<'a>(&self) -> ApiError<'a> {
        ApiError {
            ok: false,
            code: self.into(),
            message: self.to_string(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiErrorCodes::OtpExpired => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::AlreadyAuthenticated => StatusCode::BAD_REQUEST,
            ApiErrorCodes::Meow => StatusCode::IM_A_TEAPOT,
            ApiErrorCodes::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::RouteDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::Unauthenticated => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::InvalidCode => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::TotpAlreadyEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::TotpFlowNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::TotpNotEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::TotpRecoveryAlreadyUsed => StatusCode::BAD_REQUEST,
            ApiErrorCodes::SudoNotEnabled => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::SudoAlreadyEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::SudoOptionNotAvailable => StatusCode::BAD_REQUEST,
            ApiErrorCodes::AccountNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::EmailAlreadyAssociated => StatusCode::BAD_REQUEST,
            ApiErrorCodes::LoginAlreadyAssociated => StatusCode::BAD_REQUEST,
            ApiErrorCodes::WebauthnNotEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::TotpRequiredEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::WebauthnChallengeNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::WebauthnError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::Json(..) => StatusCode::BAD_REQUEST,
            ApiErrorCodes::WebauthnCompromised => StatusCode::FORBIDDEN,
            ApiErrorCodes::MailerError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::DataError(..) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorCodes::JsonSyntaxError(..) => StatusCode::BAD_REQUEST,
            ApiErrorCodes::MissingJsonContentType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ApiErrorCodes::FailedToBufferContent => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::ValidationError(..) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorCodes::AlreadyVerified => StatusCode::BAD_REQUEST,
            ApiErrorCodes::FlowNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::LoginChangeTooSoon => StatusCode::FORBIDDEN,
            ApiErrorCodes::DataNotFound(_) => StatusCode::NOT_FOUND,
            ApiErrorCodes::ActionBlocked => StatusCode::FORBIDDEN,
            ApiErrorCodes::RatelimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}

impl IntoResponse for ApiErrorCodes {
    fn into_response(self) -> axum::response::Response {
        let body = self.as_api_error();
        (self.status_code(), axum::Json(body)).into_response()
    }
}
