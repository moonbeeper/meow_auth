use std::sync::Arc;

use axum::{Extension, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs_proto::RequestChallengeResponse;

use crate::{
    auth::{
        emails::{AuthMailer, EmailVerificationCodeKind},
        otp::get_otp_code,
        sudo::{SudoOption, get_available_options},
        webauthn::get_user_passkeys,
    },
    database::models::{
        user::User,
        user_auth_challenge::{AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges},
        user_webauthn_challenge::{UserWebauthnChallenge, WebauthnChallengeKind},
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::auth_manager::AuthContext,
        v1::{auth::flows::FlowResponse, types::AuthMethod},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(otp_option))
        .routes(routes!(totp_option))
        .routes(routes!(webauthn_options))
}

/// Re-Authenticate via an OTP code
#[utoipa::path(
    post,
    path = "/",
    tags = ["sudo"],
    responses(
        (status = 200, description = "sudo re-authentication flow created", body = FlowResponse),
        (status = 400, description = "already enabled or option not available", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_option(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Otp) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let otp = get_otp_code(&global.settings);

    let login_request = UserAuthChallenges::builder()
        .user_id(Some(user.id))
        .kind(AuthChallengeKind::Otp)
        .purpose(AuthChallengePurpose::Sudo)
        .secret(Some(otp.hash))
        .session_id(Some(auth.session_id()))
        .build();

    let mut transaction = global.database.begin().await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    AuthMailer::verification_code(
        otp.code,
        EmailVerificationCodeKind::Verification,
        user.login,
        user.email,
        &global.database,
    )
    .await?;

    Ok(Json(FlowResponse {
        flow_id: login_request.id,
        next_method: vec![AuthMethod::Otp],
    }))
}

/// Re-Authenticate via a TOTP code (if enabled)
#[utoipa::path(
    post,
    path = "/totp",
    tags = ["sudo"],
    responses(
        (status = 200, description = "sudo re-authentication flow created", body = FlowResponse),
        (status = 400, description = "already enabled or option not available", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn totp_option(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Totp) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let login_request = UserAuthChallenges::builder()
        .user_id(Some(auth.user_id()))
        .kind(AuthChallengeKind::Totp)
        .purpose(AuthChallengePurpose::Sudo)
        .secret(None)
        .session_id(Some(auth.session_id()))
        .build();

    let mut transaction = global.database.begin().await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    Ok(Json(FlowResponse {
        flow_id: login_request.id,
        next_method: vec![AuthMethod::Otp],
    }))
}

/// Re-Authenticate via a Passkey
///
/// Returns the challenge for the user's browser to use to re-authenticate.
#[utoipa::path(
    post,
    path = "/webauthn",
    tags = ["sudo"],
    responses(
        (status = 200, description = "sudo re-authentication flow created"),
        (status = 400, description = "already enabled or option not available", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn webauthn_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<RequestChallengeResponse>, ApiErrorCodes> {
    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Passkey) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let passkeys = get_user_passkeys(auth.user_id(), &global.database)
        .await
        .map_err(|_| ApiErrorCodes::InternalServerError)?;

    let (client_challenge, data) = global.webauthn.start_passkey_authentication(&passkeys)?;
    let data = serde_json::to_value(data)?;

    let db_challenge = UserWebauthnChallenge::builder()
        .user_id(auth.user_id())
        .big_data(data)
        .kind(WebauthnChallengeKind::ReAuthenticate)
        .expires_at(
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.webauthn.timeout_seconds),
        )
        .build();

    let mut tx = global.database.begin().await?;
    UserWebauthnChallenge::delete_all_by_user(
        auth.user_id(),
        WebauthnChallengeKind::ReAuthenticate,
        &mut tx,
    )
    .await?;
    db_challenge.insert(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(client_challenge))
}
