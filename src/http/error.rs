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
    #[error("an error occurred while hashing a secret")]
    HashingError(#[from] argon2::password_hash::Error),
    #[error("meow *blep*")]
    Meow,
    #[error("an error occurred while interacting with the database")]
    Database(#[from] DatabaseError),
    #[error("an error occurred while interacting with the database")]
    RouteDatabase(#[from] sqlx::Error),
    #[error("an unknown error occurred")]
    InternalServerError,
    #[error("the requested session was not found")]
    SessionNotFound,
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
    #[error("Something went really wrong back there")]
    NotGood,
    /// ->
    #[error("the requested passkey was not found")]
    WebauthnNotFound,
    /// ->
    #[error("{0}")]
    ValidationError(String),
}

// wtf
impl From<()> for ApiErrorCodes {
    fn from(_: ()) -> Self {
        ApiErrorCodes::Meow
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
            ApiErrorCodes::HashingError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::Meow => StatusCode::IM_A_TEAPOT,
            ApiErrorCodes::Database(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::RouteDatabase(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::Unauthenticated => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::SessionNotFound => StatusCode::NOT_FOUND,
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
            ApiErrorCodes::NotGood => StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCodes::WebauthnNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::ValidationError(..) => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}

impl IntoResponse for ApiErrorCodes {
    fn into_response(self) -> axum::response::Response {
        let body = self.as_api_error();
        (self.status_code(), axum::Json(body)).into_response()
    }
}
