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
    #[error("an error occurred while hashing a secret")]
    HashingError(#[from] argon2::password_hash::Error),
    #[error("meow *blep*")]
    Meow,
    #[error("an error occurred while interacting with the database")]
    Database(#[from] DatabaseError),
    #[error("an error occurred while interacting with the database")]
    RouteDatabase(#[from] sqlx::Error),
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
        }
    }
}

impl IntoResponse for ApiErrorCodes {
    fn into_response(self) -> axum::response::Response {
        let body = self.as_api_error();
        (self.status_code(), axum::Json(body)).into_response()
    }
}
