use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::otp::get_otp_code,
    database::models::{
        user::User,
        user_auth_challenges::{AuthChallengeKind, UserAuthChallenges},
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::{
            auth::flows::{FlowRequest, FlowResponse},
            types::AuthMethod,
        },
    },
    job_queue::QueuedJob as _,
    mailer::{Email, MailerJob},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(otp_login))
        .routes(routes!(otp_register))
}

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "login flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_login(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Err(ApiErrorCodes::AccountNotFound);
    };

    let otp = get_otp_code().map_err(|e| {
        tracing::error!("failed generating otp code: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    let login_request = UserAuthChallenges::builder()
        .user_id(user.id)
        .kind(AuthChallengeKind::Otp)
        .secret(Some(otp.hash))
        .build();

    let mut transaction = global.database.begin().await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    let email = Email::builder()
        .text(format!("hi your code is this {}", otp.code))
        .to(user.email)
        .subject("login code".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(FlowResponse {
        flow_id: login_request.id,
        next_method: vec![AuthMethod::Otp],
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    login: String,
    email: String,
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "registration flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_register(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    // awful
    if User::find_by_email(request.email.clone(), &global.database)
        .await?
        .is_some()
    {
        return Err(ApiErrorCodes::EmailAlreadyAssociated);
    }

    if User::find_by_login(request.login.clone(), &global.database)
        .await?
        .is_some()
    {
        return Err(ApiErrorCodes::LoginAlreadyAssociated);
    }

    let otp = get_otp_code().map_err(|e| {
        tracing::error!("failed generating otp code: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    let user = User::builder()
        .email(request.email)
        .login(request.login)
        .build();

    let login_request = UserAuthChallenges::builder()
        .user_id(user.id)
        .kind(AuthChallengeKind::Otp)
        .secret(Some(otp.hash))
        .build();

    let mut transaction = global.database.begin().await?;
    user.insert(&mut transaction).await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    let email = Email::builder()
        .text(format!("hi your code is this {}", otp.code))
        .to(user.email)
        .subject("register code".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(FlowResponse {
        flow_id: login_request.id,
        next_method: vec![AuthMethod::Otp],
    }))
}
