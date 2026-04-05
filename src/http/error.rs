use axum::{http::StatusCode, response::IntoResponse};

use crate::database::error::DatabaseError;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApiError<'a> {
    code: &'a str,
    message: String,
}

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
    #[error("the provided OTP code is invalid or expired")]
    InvalidOTPCode, // TODO: should merge the use of the code stuff like a literal InvalidCode and that's it
    #[error("totp is already enabled for this account")]
    TotpAlreadyEnabled,
    #[error("the totp flow was not started before exchanging")]
    TotpFlowNotFound,
    #[error("the provided totp code is invalid")]
    TotpInvalidCode, // TODO: should merge the use of the code stuff like a literal InvalidCode and that's it
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
            ApiErrorCodes::InvalidOTPCode => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::TotpAlreadyEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::TotpFlowNotFound => StatusCode::NOT_FOUND,
            ApiErrorCodes::TotpInvalidCode => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::TotpNotEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::TotpRecoveryAlreadyUsed => StatusCode::BAD_REQUEST,
            ApiErrorCodes::SudoNotEnabled => StatusCode::UNAUTHORIZED,
            ApiErrorCodes::SudoAlreadyEnabled => StatusCode::BAD_REQUEST,
            ApiErrorCodes::SudoOptionNotAvailable => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for ApiErrorCodes {
    fn into_response(self) -> axum::response::Response {
        let body = self.as_api_error();
        (self.status_code(), axum::Json(body)).into_response()
    }
}
