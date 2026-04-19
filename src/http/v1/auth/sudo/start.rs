use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::Passkey;
use webauthn_rs_proto::RequestChallengeResponse;

use crate::{
    auth::{
        otp::get_otp_code,
        sudo::{SudoOption, get_available_options},
    },
    database::models::{
        user::User,
        user_auth_challenges::{AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges},
        user_webauthn::UserWebauthn,
        user_webauthn_challenges::{UserWebauthnChallenge, WebauthnChallengeKind},
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::{auth::flows::FlowResponse, types::AuthMethod},
    },
    job_queue::QueuedJob as _,
    mailer::{Email, MailerJob},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(otp_option))
        .routes(routes!(totp_option))
        .routes(routes!(webauthn_options))
}

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "login flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_option(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Otp) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let otp = get_otp_code().map_err(|e| {
        tracing::error!("failed generating otp code: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    let login_request = UserAuthChallenges::builder()
        .user_id(user.id)
        .kind(AuthChallengeKind::Otp)
        .purpose(AuthChallengePurpose::Sudo)
        .secret(Some(otp.hash))
        .session_id(Some(auth.session_id()))
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

#[utoipa::path(
    post,
    path = "/totp",
    responses(
        (status = 200, description = "login flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn totp_option(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Totp) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let login_request = UserAuthChallenges::builder()
        .user_id(auth.user_id())
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

#[utoipa::path(
    post,
    path = "/webauthn",
    responses(
        (status = 200, description = "login flow created"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn webauthn_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<RequestChallengeResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&SudoOption::Passkey) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let Ok(passkeys) = UserWebauthn::find_many_by_user_id(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::Meow);
    };
    let passkeys: Vec<_> = passkeys
        .into_iter()
        .flat_map(|v| serde_json::from_value::<Passkey>(v.big_data))
        .collect();

    let (client_challenge, data) = global.webauthn.start_passkey_authentication(&passkeys)?;

    let data = serde_json::to_value(data)?;
    let db_challenge = UserWebauthnChallenge::builder()
        .user_id(auth.user_id())
        .big_data(data)
        .kind(WebauthnChallengeKind::Authenticate)
        .expires_at(
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.webauthn.timeout_seconds),
        )
        .build();

    let mut tx = global.database.begin().await?;
    UserWebauthnChallenge::delete_all_by_user(
        auth.user_id(),
        WebauthnChallengeKind::Authenticate,
        &mut tx,
    )
    .await?;
    db_challenge.insert(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(client_challenge))
}
