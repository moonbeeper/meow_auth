use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    auth::{
        email::{get_token, hash_token},
        flags::UserFlag,
        mailer::{AuthMailer, NewEmailVerificationCodeKind},
    },
    database::models::{user::User, user_email_mod_request::UserEmailModificationRequest},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{auth_manager::AuthContext, require_user_flag::RequireUserFlagLayer},
        v1::types::AlrightResponse,
        validator::Valid,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(change_user_email))
        .routes(routes!(exchange_change_user_email))
        .layer(RequireUserFlagLayer::new().forbid(UserFlag::CannotModifyEmail))
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ChangeEmailRequest {
    #[validate(custom(function = "crate::auth::valid_email"))]
    email: String,
}

/// Change your current email address
///
/// This will send a verification email to both your current and new email addresses. You must verify both emails to complete the change.
#[utoipa::path(
    patch,
    path = "/email",
    tags = ["user"],
    responses(
        (status = 200, description = "successfully created the change request", body = AlrightResponse),
        (status = 401, description = "sudo not enabled", body = ApiError),
        (status = 400, description = "new email is already associated", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn change_user_email(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Valid(Json(request)): Valid<Json<ChangeEmailRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let Ok(None) = User::find_by_email(request.email.clone(), &global.database).await else {
        return Err(ApiErrorCodes::EmailAlreadyAssociated);
    };

    let current_email = get_token(&global.settings);
    let new_email = get_token(&global.settings);

    let email_request = UserEmailModificationRequest::builder()
        .user_id(auth.user_id())
        .current_email(user.email.clone())
        .new_email(request.email.clone())
        .current_email_token(current_email.hash)
        .new_email_token(new_email.hash)
        .build();

    let mut tx = global.database.begin().await?;
    UserEmailModificationRequest::delete_all_by_user(auth.user_id(), &mut tx).await?;
    email_request.insert(&mut tx).await?;
    audit::log(
        auth.user_id(),
        AuditAction::EmailChangeRequested,
        None,
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    AuthMailer::email_verification(
        NewEmailVerificationCodeKind::Current,
        current_email.token,
        request.email.clone(),
        user.login.clone(),
        user.email,
        &global.database,
    )
    .await?;

    AuthMailer::email_verification(
        NewEmailVerificationCodeKind::New,
        new_email.token,
        request.email.clone(),
        user.login,
        request.email,
        &global.database,
    )
    .await?;

    Ok(Json(AlrightResponse::default()))
}

#[derive(Debug, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
pub struct ExchangeChangeEmailRequest {
    #[validate(length(equal = 32, message = "token must be 32 characters long"))]
    token: String,
}

// for whatever reason I was wondering if I had written "change" correctly because it really sounded bad in my head.
/// Exchange the Email Change Token
#[utoipa::path(
    post,
    path = "/email",
    tags = ["user"],
    responses(
        (status = 200, description = "successfully verified email change request", body = AlrightResponse),
        (status = 404, description = "email change flow not found", body = ApiError),
        (status = 400, description = "email already verified", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange_change_user_email(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Valid(Json(request)): Valid<Json<ExchangeChangeEmailRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let hashed_token = hash_token(&request.token, &global.settings);

    let Ok(Some(mut email_request)) = UserEmailModificationRequest::find_by_token_and_user_id(
        &hashed_token,
        auth.user_id(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::FlowNotFound);
    };

    let was_current = email_request.current_email_token == hashed_token;

    if (was_current && email_request.current_email_verified)
        || (!was_current && email_request.new_email_verified)
    {
        return Err(ApiErrorCodes::AlreadyVerified);
    }

    if was_current {
        email_request.current_email_verified = true;
    } else {
        email_request.new_email_verified = true;
    }

    if email_request.current_email_verified && email_request.new_email_verified {
        user.email = email_request.new_email.clone();
        user.email_verified = true;

        let mut tx = global.database.begin().await?;
        user.update(&mut tx).await?;
        email_request.delete(&mut tx).await?;
        audit::log(auth.user_id(), AuditAction::EmailChanged, None, &mut tx).await?;
        tx.commit().await?;

        AuthMailer::email_updated(
            NewEmailVerificationCodeKind::Current,
            email_request.new_email.clone(),
            user.login.clone(),
            email_request.current_email,
            &global.database,
        )
        .await?;

        AuthMailer::email_updated(
            NewEmailVerificationCodeKind::New,
            email_request.new_email.clone(),
            user.login,
            email_request.new_email,
            &global.database,
        )
        .await?;

        return Ok(Json(AlrightResponse::default()));
    }

    let mut tx = global.database.begin().await?;
    email_request.update(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
