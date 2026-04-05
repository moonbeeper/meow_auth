use std::sync::Arc;

use argon2::{Argon2, PasswordHasher as _, PasswordVerifier as _};
use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{
        sudo::{SudoOption, enable_sudo_tx, get_available_options},
        totp::{decrypt_secrets, is_recovery_code_used, make_totp, set_recovery_code_used},
    },
    database::{
        id::UlidId,
        models::{
            user::User,
            user_auth_challenges::{AuthChallengePurpose, UserAuthChallenges},
            user_totp::UserTotp,
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::auth::flows::generate_otp_code,
    },
    job_queue::QueuedJob as _,
    mailer::{Email, MailerJob},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(get_enable_options))
        .routes(routes!(enable_sudo))
        .routes(routes!(enable_sudo_exchange))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct SudoOptionsResponse {
    options: Vec<SudoOption>,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "sudo enable options", body = SudoOptionsResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn get_enable_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<SudoOptionsResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let options = get_available_options(auth.user_id(), &global.database).await;

    Ok(Json(SudoOptionsResponse { options }))
}

// do work like send OTP -> exchange

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct EnableSudoRequest {
    option: SudoOption,
}

// TODO: merge with login flow.
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct FlowResponse {
    flow_id: UlidId,
}

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "sudo flow id for enable exchange", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn enable_sudo(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<EnableSudoRequest>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    if !sudo_options.contains(&request.option) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    let mut secret = None;
    if request.option == SudoOption::Otp {
        let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
            return Err(ApiErrorCodes::InvalidOTPCode);
        };

        let code = generate_otp_code();
        let argon2 = Argon2::default();
        let code_hash = argon2.hash_password(code.as_bytes())?.to_string();
        let email = Email::builder()
            .text(format!("hi your code is this {}", code.to_uppercase()))
            .to(user.email)
            .subject("your verification code".to_string())
            .build();

        MailerJob::dispatch(&global.database, email)
            .await
            .map_err(|e| {
                tracing::error!("failed dispatching job: {e}");
                ApiErrorCodes::InternalServerError
            })?;

        secret = Some(code_hash)
    }

    let challenge = UserAuthChallenges::builder()
        .user_id(auth.user_id())
        .session_id(Some(auth.session_id()))
        .kind(request.option.into())
        .secret(secret)
        .purpose(AuthChallengePurpose::Sudo)
        .build();

    let mut transaction = global.database.begin().await?;
    challenge.insert(&mut transaction).await?;
    transaction.commit().await?;

    Ok(Json(FlowResponse {
        flow_id: challenge.id,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct EnableSudoExchangeRequest {
    flow_id: UlidId,
    code: String,
}

#[utoipa::path(
    post,
    path = "/exchange",
    responses(
        (status = 200, description = "sudo enabled successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn enable_sudo_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<EnableSudoExchangeRequest>,
) -> Result<(), ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    if auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    }

    let Ok(Some(flow)) = UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidOTPCode);
    };

    if flow.purpose != AuthChallengePurpose::Sudo {
        return Err(ApiErrorCodes::InvalidOTPCode);
    }

    let now = chrono::Utc::now();
    if flow.expires_at < now {
        return Err(ApiErrorCodes::InvalidOTPCode);
    }

    if flow.session_id != Some(auth.session_id()) {
        return Ok(());
    }
    // let mut transaction = global.database.begin().await?;
    // flow.state = AuthChallengeState::Completed;
    // flow.update(&mut transaction).await?;
    // transaction.commit().await?;

    let sudo_options = get_available_options(auth.user_id(), &global.database).await;
    let sudo_option = SudoOption::from(flow.kind);
    if !sudo_options.contains(&sudo_option) {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    }

    match sudo_option {
        SudoOption::Otp => {
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
        }
        SudoOption::Totp => {
            let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
                return Err(ApiErrorCodes::InvalidOTPCode);
            };

            if !user.totp_enabled {
                return Err(ApiErrorCodes::InternalServerError);
            }

            let Ok(Some(mut db_totp)) = UserTotp::find_one_by_user(user.id, &global.database).await
            else {
                return Err(ApiErrorCodes::InternalServerError);
            };

            let encrypted_secrets = db_totp.clone().into();
            let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
                tracing::error!("something went wrong while decrypting totp secrets: {e}");
                ApiErrorCodes::InternalServerError
            })?;

            // TODO: regex for recovery codes
            if request.code.len() == 6 {
                let totp_client = make_totp(user.login.clone(), totp.secret, &global.settings)
                    .map_err(|e| {
                        tracing::error!("something went wrong while creating the totp client: {e}");
                        ApiErrorCodes::InternalServerError
                    })?;

                if !totp_client.check_current(&request.code).unwrap_or(false) {
                    return Err(ApiErrorCodes::TotpInvalidCode);
                }

                let mut tx = global.database.begin().await?;
                db_totp.update(&mut tx).await?;
                tx.commit().await?;
            } else {
                // TODO: should merge some stuff bruh
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

                let email = Email::builder()
                    .text("hi you used a recovery code. have a great great night".to_string())
                    .to(user.email.clone())
                    .subject("totp recovery code used".to_string())
                    .build();

                MailerJob::dispatch(&global.database, email)
                    .await
                    .map_err(|e| {
                        tracing::error!("failed dispatching job: {e}");
                        ApiErrorCodes::InternalServerError
                    })?;
            }
        }
        SudoOption::Passkey => todo!(),
    };

    enable_sudo_tx(auth.session_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed enabling sudo: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(())
}
