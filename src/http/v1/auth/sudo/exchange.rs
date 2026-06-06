use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::PasskeyAuthentication;
use webauthn_rs_proto::PublicKeyCredential;

use crate::{
    auth::{
        emails::AuthMailer,
        otp::verify_otp_code,
        sudo::{enable_sudo_tx, has_sudo_option, is_flow_correct},
        totp::{decrypt_secrets, get_totp, is_recovery_code_used, set_recovery_code_used},
        webauthn::update_passkey_with_authentication_result,
    },
    database::{
        id::UlidId,
        models::{
            user::User,
            user_auth_challenge::{AuthChallengeState, UserAuthChallenges},
            user_totp::UserTotp,
            user_webauthn::UserWebauthn,
            user_webauthn_challenge::{UserWebauthnChallenge, WebauthnChallengeKind},
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::types::{AlrightResponse, AuthenticationPasskeyRequest},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(exchange))
        .routes(routes!(webauthn_exchange))
        .routes(routes!(totp_exchange))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ExchangeRequest {
    flow_id: UlidId,
    code: String,
}

/// Exchange the flow to enable sudo via an OTP code
///
/// This uses the flow id provided by the start method
#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "sudo enable exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<ExchangeRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    if !is_flow_correct(&flow, auth.session_id()) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // short circuit if the user doesn't have that sudo option
    if !has_sudo_option(flow.kind.into(), auth.user_id(), &global.database).await {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let secret_hash = flow
        .secret
        .as_ref()
        .ok_or_else(|| ApiErrorCodes::OtpExpired)?;

    if !verify_otp_code(&request.code, secret_hash) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // mark flow as completed
    let mut transaction = global.database.begin().await?;
    flow.state = AuthChallengeState::Completed;
    flow.update(&mut transaction).await?;
    transaction.commit().await?;

    enable_sudo_tx(auth.session_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed enabling sudo: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(AlrightResponse::default()))
}

/// Exchange the flow to enable sudo via a Passkey
#[utoipa::path(
    post,
    path = "/webauthn",
    responses(
        (status = 200, description = "sudo enable exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn webauthn_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<AuthenticationPasskeyRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let request: PublicKeyCredential = request
        .try_into()
        .map_err(|_| ApiErrorCodes::WebauthnChallengeNotFound)?;

    let Ok(Some(db_challenge)) = UserWebauthnChallenge::take_by_user_id(
        auth.user_id(),
        WebauthnChallengeKind::Authenticate,
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::Unauthenticated);
    };

    let challenge: PasskeyAuthentication = serde_json::from_value(db_challenge.big_data)?;

    let auth_result = global
        .webauthn
        .finish_passkey_authentication(&request, &challenge)?;

    let Ok(Some(mut passkey)) =
        UserWebauthn::find_by_credential_id(auth_result.cred_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    // check counter to account for cloning attackssss
    if auth_result.counter() <= passkey.counter as u32 {
        // TODO: cloning attack. deactivate passkey and notify via email
        return Err(ApiErrorCodes::WebauthnCompromised);
    }
    if passkey.user_id != user.id {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    }
    update_passkey_with_authentication_result(&mut passkey, &auth_result)
        .map_err(|_| ApiErrorCodes::InternalServerError)?;

    // update passkey
    let mut tx = global.database.begin().await?;
    passkey.counter += 1;
    passkey.update(&mut tx).await?;
    tx.commit().await?;

    enable_sudo_tx(auth.session_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed enabling sudo: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(AlrightResponse::default()))
}

/// Exchange the flow to enable sudo via a TOTP code
///
/// This uses the flow id provided by the start method
#[utoipa::path(
    post,
    path = "/totp",
    responses(
        (status = 200, description = "sudo enable exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn totp_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<ExchangeRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    if !is_flow_correct(&flow, auth.session_id()) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // short circuit if the user doesn't exist
    let Ok(Some(user)) = User::find_by_id(flow.user_id.unwrap(), &global.database).await else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    if !user.totp_enabled {
        return Err(ApiErrorCodes::InternalServerError);
    }

    let Ok(Some(mut db_totp)) = UserTotp::find_one_by_user(user.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    // TODO: regex for recovery codes
    if request.code.len() == 6 {
        let totp_client =
            get_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
                tracing::error!("something went wrong while creating the totp client: {e}");
                ApiErrorCodes::InternalServerError
            })?;

        if !totp_client.check_current(&request.code).unwrap_or(false) {
            return Err(ApiErrorCodes::InvalidCode);
        }

        let mut tx = global.database.begin().await?;
        db_totp.update(&mut tx).await?;
        tx.commit().await?;
    } else {
        let (idx, check) =
            is_recovery_code_used(&db_totp, &totp.recovery_secret, request.code.clone());
        if check {
            return Err(ApiErrorCodes::TotpRecoveryAlreadyUsed);
        }

        set_recovery_code_used(idx, &mut db_totp, &global.database)
            .await
            .map_err(|e| {
                tracing::error!(
                    "something went wrong while setting the used totp recovery code: {e}"
                );
                ApiErrorCodes::InternalServerError
            })?;

        AuthMailer::totp_recovery_code_used(
            user.login.clone(),
            user.email.clone(),
            &global.database,
        )
        .await?;
    }

    let mut transaction = global.database.begin().await?;
    flow.state = AuthChallengeState::Completed;
    flow.update(&mut transaction).await?;
    transaction.commit().await?;

    enable_sudo_tx(auth.session_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed enabling sudo: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(AlrightResponse::default()))
}
