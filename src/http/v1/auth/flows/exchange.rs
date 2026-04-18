use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::{Passkey, PasskeyAuthentication};
use webauthn_rs_proto::PublicKeyCredential;

use crate::{
    auth::{
        otp::verify_otp_code,
        session::{create_session, create_session_cookie},
        totp::{decrypt_secrets, get_totp, is_recovery_code_used, set_recovery_code_used},
        webauthn::get_challenge_id_from_cookies,
    },
    database::{
        id::UlidId,
        models::{
            user::User,
            user_auth_challenges::{AuthChallengeKind, AuthChallengeState, UserAuthChallenges},
            user_totp::UserTotp,
            user_webauthn::UserWebauthn,
            user_webauthn_challenges::{UserWebauthnChallenge, WebauthnChallengeKind},
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::{
            auth::flows::FlowResponse,
            types::{AlrightResponse, AuthMethod, AuthenticationPasskeyRequest, RouteEither},
        },
    },
    job_queue::QueuedJob as _,
    mailer::{Email, MailerJob},
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

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "login exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<ExchangeRequest>,
) -> Result<RouteEither<Json<FlowResponse>, Json<AlrightResponse>>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    let now = chrono::Utc::now();
    if flow.expires_at < now {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // short circuit if the user doesn't exist
    let Ok(Some(user)) = User::find_by_id(flow.user_id, &global.database).await else {
        return Err(ApiErrorCodes::InvalidCode);
    };

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

    // swap into next step if totp is enabled
    if user.totp_enabled {
        let login_request = UserAuthChallenges::builder()
            .user_id(user.id)
            .kind(AuthChallengeKind::Totp)
            .build();

        let mut transaction = global.database.begin().await?;
        login_request.insert(&mut transaction).await?;
        transaction.commit().await?;

        return Ok(RouteEither::Left(Json(FlowResponse {
            flow_id: login_request.id,
            next_method: vec![AuthMethod::Totp],
        })));
    }

    let session_id = create_session(user.id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id, &cookies, &global.settings);

    let email = Email::builder()
        .text("hi you opened a new session :)".to_string())
        .to(user.email)
        .subject("new session".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(RouteEither::Right(Json(AlrightResponse::default())))
}

#[utoipa::path(
    post,
    path = "/webauthn",
    responses(
        (status = 200, description = "login exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn webauthn_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<AuthenticationPasskeyRequest>,
) -> Result<RouteEither<Json<FlowResponse>, Json<AlrightResponse>>, ApiErrorCodes> {
    let request: PublicKeyCredential =
        request.try_into().map_err(|_| ApiErrorCodes::InvalidCode)?;

    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Some(challenge_id) = get_challenge_id_from_cookies(&cookies, &global.settings) else {
        return Err(ApiErrorCodes::SudoAlreadyEnabled);
    };

    let Ok(Some(db_challenge)) = UserWebauthnChallenge::find_by_id(
        challenge_id,
        WebauthnChallengeKind::Authenticate,
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    };

    let Ok(Some(user)) = User::find_by_id(db_challenge.user_id, &global.database).await else {
        return Err(ApiErrorCodes::Unauthenticated);
    };

    let mut tx = global.database.begin().await.unwrap();
    db_challenge.delete(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let challenge: PasskeyAuthentication = serde_json::from_value(db_challenge.big_data).unwrap();

    let auth_result = global
        .webauthn
        .finish_passkey_authentication(&request, &challenge)
        .unwrap(); // todo: handle errors properly

    let Ok(Some(mut passkey)) =
        UserWebauthn::find_by_credential_id(auth_result.cred_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    // check counter to account for cloning attackssss
    if auth_result.counter() <= passkey.counter as u32 {
        return Err(ApiErrorCodes::EmailAlreadyAssociated);
    }
    if passkey.user_id != user.id {
        return Err(ApiErrorCodes::Meow);
    }

    if auth_result.needs_update() {
        let mut big_data: Passkey = serde_json::from_value(passkey.big_data).unwrap();
        big_data.update_credential(&auth_result);
        passkey.big_data = serde_json::to_value(big_data).unwrap();
    }

    let mut tx = global.database.begin().await.unwrap();
    passkey.update(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let session_id = create_session(passkey.user_id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: you opened a new{}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id, &cookies, &global.settings);

    let email = Email::builder()
        .text("hi you opened a new session via webauthn :)".to_string())
        .to(user.email)
        .subject("new session".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(RouteEither::Right(Json(AlrightResponse::default())))
}

#[utoipa::path(
    post,
    path = "/totp",
    responses(
        (status = 200, description = "login exchanged successfully"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn totp_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<ExchangeRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    let now = chrono::Utc::now();
    if flow.expires_at < now {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // short circuit if the user doesn't exist
    let Ok(Some(user)) = User::find_by_id(flow.user_id, &global.database).await else {
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

    let mut transaction = global.database.begin().await?;
    flow.state = AuthChallengeState::Completed;
    flow.update(&mut transaction).await?;
    transaction.commit().await?;

    let session_id = create_session(user.id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id, &cookies, &global.settings);

    let email = Email::builder()
        .text("hi you opened a new session :)".to_string())
        .to(user.email)
        .subject("new session".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(AlrightResponse::default()))
}
