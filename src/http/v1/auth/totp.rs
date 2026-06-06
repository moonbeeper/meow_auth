use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{
        emails::AuthMailer,
        totp::{create_user_totp, decrypt_secrets, get_totp, usable_recovery_codes},
    },
    database::models::{user::User, user_totp::UserTotp as DbUserTotp},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::AlrightResponse,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(create_totp_options))
        .routes(routes!(exchange_totp_creation))
        .routes(routes!(disable_totp))
        .routes(routes!(see_recovery_codes))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CreateTotpResponse {
    uri: String,
    secret: String,
    recovery_codes: Vec<String>,
}

/// Get the TOTP creation options
#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "totp relevant info", body = CreateTotpResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn create_totp_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<CreateTotpResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if user.totp_enabled {
        return Err(ApiErrorCodes::TotpAlreadyEnabled);
    }

    let mut tx = global.database.begin().await?;
    DbUserTotp::delete_all_by_user(auth.user_id(), &mut tx).await?;
    tx.commit().await?;

    let totp = create_user_totp(auth.user_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("something went wrong while creating the user totp: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    let totp_client = get_totp(user.login, totp.secret.clone(), &global.settings).map_err(|e| {
        tracing::error!("something went wrong while creating the totp client: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let uri = totp_client.get_url();

    Ok(Json(CreateTotpResponse {
        uri,
        secret: totp.secret,
        recovery_codes: totp.recovery_codes,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct VerifyTotpRequest {
    code: String,
}

/// Exchange the TOTP creation options to enable TOTP
///
/// This will make use of the Six Digit code generated on your authenticator
#[utoipa::path(
    post,
    path = "/exchange",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "totp successfully enabled"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange_totp_creation(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if user.totp_enabled {
        return Err(ApiErrorCodes::TotpAlreadyEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpFlowNotFound);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client = get_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while creating the totp client: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let mut tx = global.database.begin().await?;
    user.totp_enabled = true;
    user.update(&mut tx).await?;
    db_totp.update(&mut tx).await?;
    tx.commit().await?;

    AuthMailer::totp_enabled(user.login, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}

/// Disable TOTP
///
/// This will make use of the Six Digit code generated on your authenticator
#[utoipa::path(
    delete,
    path = "/",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "totp successfully disabled"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn disable_totp(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if !user.totp_enabled {
        return Err(ApiErrorCodes::TotpNotEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpNotEnabled);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client = get_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while creating the totp client: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let mut tx = global.database.begin().await?;
    user.totp_enabled = false;
    user.update(&mut tx).await?;
    db_totp.delete(&mut tx).await?;
    tx.commit().await?;

    AuthMailer::totp_disabled(user.login, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RecoveryCodesTotpResponse {
    recovery_codes: Vec<String>,
}

/// Get your TOTP recovery codes
#[utoipa::path(
    post,
    path = "/recovery",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "usable totp recovery codes"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn see_recovery_codes(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<Json<RecoveryCodesTotpResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if !user.totp_enabled {
        return Err(ApiErrorCodes::TotpNotEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpNotEnabled);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client = get_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while creating the totp client: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let recovery_codes = usable_recovery_codes(&db_totp, &totp.recovery_secret);

    let mut tx = global.database.begin().await?;
    db_totp.update(&mut tx).await?;
    tx.commit().await?;

    AuthMailer::totp_recovery_codes_seen(user.login, user.email, &global.database).await?;

    Ok(Json(RecoveryCodesTotpResponse { recovery_codes }))
}
