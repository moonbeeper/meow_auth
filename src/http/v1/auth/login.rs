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
    http::middleware::auth_manager::AuthContext,
};

// TODO: should have validation of these things
// TODO: should have a big monolith error enum
// TODO: should let the user log in via their username, maybe by using a regex to determine if the input is an email or login

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    email: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct LoginResponse {
    flow_id: UlidId,
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "login flow created", body = LoginResponse),
    )
)]
pub async fn login(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Option<LoginResponse>>, ()> {
    if auth.is_authenticated() {
        return Ok(Json(None));
    }

    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Ok(Json(None));
    };

    // huh
    let code: String = rand::rng()
        .sample_iter(Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let argon2 = Argon2::default();
    let code_hash = argon2
        .hash_password(code.to_uppercase().as_bytes())
        .map_err(|e| {
            tracing::error!("failed hashing code: {}", e);
        })?
        .to_string();

    let login_request = UserLoginRequest::builder()
        .user_id(user.id)
        .kind(LoginFlowKind::Otp)
        .secret(Some(code_hash))
        .expires_at(chrono::Utc::now() + chrono::Duration::minutes(10))
        .build();
    let mut transaction = global.database.begin().await.map_err(|e| {
        tracing::error!("failed starting transaction: {}", e);
    })?;
    login_request.insert(&mut transaction).await.map_err(|e| {
        tracing::error!("failed inserting login request: {}", e);
    })?;
    transaction.commit().await.map_err(|e| {
        tracing::error!("failed committing transaction: {}", e);
    })?;

    global
        .mailer
        .mail(
            user.email,
            format!("hi your code is this {}", code.to_uppercase()),
        )
        .await
        .map_err(|e| {
            tracing::error!("failed sending mail: {}", e);
        })?;

    Ok(Json(Some(LoginResponse {
        flow_id: login_request.id,
    })))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    email: String,
    login: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    flow_id: UlidId,
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "registration flow created", body = RegisterResponse),
    )
)]
pub async fn register(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<Option<RegisterResponse>>, ()> {
    if auth.is_authenticated() {
        return Ok(Json(None));
    }

    if User::find_by_email_and_login(
        request.email.clone(),
        request.login.clone(),
        &global.database,
    )
    .await
    .map_err(|e| tracing::error!("woops from database: {}", e))?
    .is_some()
    {
        return Ok(Json(None));
    }

    let code: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let argon2 = Argon2::default();
    let code_hash = argon2
        .hash_password(code.to_uppercase().as_bytes())
        .map_err(|e| {
            tracing::error!("failed hashing code: {}", e);
        })?
        .to_string();

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

    let mut transaction = global.database.begin().await.map_err(|e| {
        tracing::error!("failed starting transaction: {}", e);
    })?;
    user.insert(&mut transaction).await.map_err(|e| {
        tracing::error!("failed inserting new user: {}", e);
    })?;
    login_request.insert(&mut transaction).await.map_err(|e| {
        tracing::error!("failed inserting login request: {}", e);
    })?;
    transaction.commit().await.map_err(|e| {
        tracing::error!("failed committing transaction: {}", e);
    })?;

    // todo: holy shit this takes ages if done this way. must have next a queue worker for this.
    global
        .mailer
        .mail(
            user.email.clone(),
            format!("hi your code is this {}", code.to_uppercase()),
        )
        .await
        .map_err(|e| {
            tracing::error!("failed sending mail: {}", e);
        })?;
    tracing::info!("is this maybe?");

    Ok(Json(Some(RegisterResponse {
        flow_id: login_request.id,
    })))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ExchangeRequest {
    flow_id: UlidId,
    code: String,
}

#[utoipa::path(
    post,
    path = "/exchange",
    request_body = ExchangeRequest
)]
pub async fn exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<ExchangeRequest>,
) -> Result<(), ()> {
    if auth.is_authenticated() {
        return Ok(());
    }

    let Ok(Some(mut flow)) = UserLoginRequest::find_by_id(request.flow_id, &global.database).await
    else {
        return Ok(());
    };

    let now = chrono::Utc::now();
    if flow.expires_at < now
        || flow.state != LoginFlowState::Pending
        || flow.kind != LoginFlowKind::Otp
    {
        return Ok(());
    }

    let argon2 = Argon2::default();
    let code = request.code.to_uppercase();
    let secret_hash = flow.secret.as_deref().unwrap_or("");
    let Ok(parsed_hash) = argon2::PasswordHash::new(secret_hash) else {
        return Ok(());
    };

    let is_valid = argon2
        .verify_password(code.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Ok(());
    }

    let mut transaction = global.database.begin().await.map_err(|e| {
        tracing::error!("failed starting transaction: {}", e);
    })?;
    flow.state = LoginFlowState::Completed;
    flow.update(&mut transaction).await.map_err(|e| {
        tracing::error!("failed setting the flow state to completed: {}", e);
    })?;
    transaction.commit().await.map_err(|e| {
        tracing::error!("failed committing transaction: {}", e);
    })?;

    let Ok(Some(user)) = User::find_by_id(flow.user_id, &global.database).await else {
        return Ok(());
    };

    let session_id = create_session(user.id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
        })?;
    create_session_cookie(session_id.to_string(), &cookies, &global.settings);

    Ok(())
}
