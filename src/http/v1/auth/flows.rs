use std::sync::Arc;

use argon2::{Argon2, PasswordHasher, PasswordVerifier as _};
use axum::{Extension, Json, extract::State};
use rand::{RngExt, distr::Alphanumeric};
use tower_cookies::Cookies;

use crate::{
    auth::session::{create_session, create_session_cookie},
    database::{
        id::UlidId,
        models::{
            user::User,
            user_login_request::{LoginFlowKind, LoginFlowState, UserLoginRequest},
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
    },
    job_queue::QueuedJob,
    mailer::{Email, MailerJob},
};

// TODO: should use correctly errors.
// TODO: should have validation of these things
// TODO: should let the user log in via their username, maybe by using a regex to determine if the input is an email or login

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    email: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowResponse {
    flow_id: UlidId,
}

#[utoipa::path(
    post,
    path = "/",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "login flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn login(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Option<FlowResponse>>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Ok(Json(None));
    };

    // huh
    let code = generate_otp_code();
    let argon2 = Argon2::default();
    let code_hash = argon2
        .hash_password(code.to_uppercase().as_bytes())?
        .to_string();

    let login_request = UserLoginRequest::builder()
        .user_id(user.id)
        .kind(LoginFlowKind::Otp)
        .secret(Some(code_hash))
        .expires_at(chrono::Utc::now() + chrono::Duration::minutes(10))
        .build();

    let mut transaction = global.database.begin().await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    let email = Email::builder()
        .text(format!("hi your code is this {}", code.to_uppercase()))
        .to(user.email)
        .subject("login code".to_string())
        .build();

    MailerJob::dispatch(global, email).await.map_err(|e| {
        tracing::error!("failed dispatching job: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    Ok(Json(Some(FlowResponse {
        flow_id: login_request.id,
    })))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    email: String,
    login: String,
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
pub async fn register(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<Option<FlowResponse>>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    if User::find_by_email_and_login(
        request.email.clone(),
        request.login.clone(),
        &global.database,
    )
    .await?
    .is_some()
    {
        return Ok(Json(None));
    }

    let code = generate_otp_code();
    let argon2 = Argon2::default();
    let code_hash = argon2.hash_password(code.as_bytes())?.to_string();

    let user = User::builder()
        .email(request.email)
        .login(request.login)
        .build();

    let login_request = UserLoginRequest::builder()
        .user_id(user.id)
        .kind(LoginFlowKind::Otp)
        .secret(Some(code_hash))
        .expires_at(chrono::Utc::now() + chrono::Duration::minutes(10))
        .build();

    let mut transaction = global.database.begin().await?;
    user.insert(&mut transaction).await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    let email = Email::builder()
        .text(format!("hi your code is this {}", code))
        .to(user.email)
        .subject("register code".to_string())
        .build();

    MailerJob::dispatch(global, email).await.map_err(|e| {
        tracing::error!("failed dispatching job: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    Ok(Json(Some(FlowResponse {
        flow_id: login_request.id,
    })))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ExchangeRequest {
    flow_id: UlidId,
    code: String,
}

// TODO: uhhh sometimes this straight up refuses to work? what
#[utoipa::path(
    post,
    path = "/exchange",
    request_body = ExchangeRequest,
    responses(
        (status = 200, description = "code exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<ExchangeRequest>,
) -> Result<(), ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(Some(mut flow)) = UserLoginRequest::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidOTPCode);
    };

    let now = chrono::Utc::now();
    if flow.expires_at < now
        || flow.state != LoginFlowState::Pending
        || flow.kind != LoginFlowKind::Otp
    {
        return Err(ApiErrorCodes::InvalidOTPCode);
    }

    let mut transaction = global.database.begin().await?;
    flow.state = LoginFlowState::Completed;
    flow.update(&mut transaction).await?;
    transaction.commit().await?;

    let argon2 = Argon2::default();
    let code = request.code.to_uppercase();
    let secret_hash = flow
        .secret
        .as_ref()
        .ok_or_else(|| ApiErrorCodes::OtpExpired)?;
    let Ok(parsed_hash) = argon2::PasswordHash::new(secret_hash) else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if argon2
        .verify_password(code.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(ApiErrorCodes::InvalidOTPCode);
    }

    let Ok(Some(user)) = User::find_by_id(flow.user_id, &global.database).await else {
        return Err(ApiErrorCodes::InvalidOTPCode);
    };

    let session_id = create_session(user.id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id.to_string(), &cookies, &global.settings);

    Ok(())
}

fn generate_otp_code() -> String {
    let code: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    code.to_uppercase()
}
