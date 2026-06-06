use std::sync::Arc;

use axum::{Extension, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::PasskeyAuthentication;
use webauthn_rs_proto::PublicKeyCredential;

use crate::{
    auth::{
        emails::AuthMailer,
        otp::{is_flow_correct, verify_otp_code},
        session::{create_session, create_session_cookie},
        totp::{decrypt_secrets, get_totp, is_recovery_code_used, set_recovery_code_used},
        webauthn::{get_challenge_id_from_cookies, update_passkey_with_authentication_result},
    },
    database::{
        id::UlidId,
        models::{
            user::User,
            user_auth_challenge::{
                AuthChallengeKind, AuthChallengePurpose, AuthChallengeState, UserAuthChallenges,
            },
            user_signup::UserSignup,
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
        v1::{
            auth::flows::FlowResponse,
            types::{AlrightResponse, AuthMethod, AuthenticationPasskeyRequest, RouteEither},
        },
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

/// Exchange the flow to login via an OTP code
///
/// This uses the flow id provided by the start method.
/// This might return a next_method of TOTP if the user has TOTP enabled
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

    if !is_flow_correct(&flow) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let mut pre_user: Option<User> = None;

    // dont short circuit if the purpose is signup. if it isn't, short circuit if the user doesn't exist
    if flow.purpose != AuthChallengePurpose::Signup {
        let Ok(Some(db_user)) = User::find_by_id(flow.user_id.unwrap(), &global.database).await
        else {
            return Err(ApiErrorCodes::InvalidCode);
        };
        pre_user = Some(db_user)
    }

    let secret_hash = flow
        .secret
        .as_ref()
        .ok_or_else(|| ApiErrorCodes::OtpExpired)?;

    if !verify_otp_code(&request.code, secret_hash) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    if flow.purpose == AuthChallengePurpose::Signup {
        let mut tx = global.database.begin().await?;

        let Ok(Some(db_user)) = UserSignup::take_by_id(flow.user_signup_id.unwrap(), &mut tx).await
        else {
            return Err(ApiErrorCodes::InvalidCode);
        };

        let user = User::builder()
            .login(db_user.login.clone())
            .email(db_user.email.clone())
            .email_verified(true)
            .build();

        db_user.delete_all_by_email_and_login(&mut tx).await?;
        user.insert(&mut tx).await?;
        tx.commit().await?;
        pre_user = Some(user);
    }

    let user = pre_user.unwrap();

    // mark flow as completed
    let mut transaction = global.database.begin().await?;
    flow.state = AuthChallengeState::Completed;
    flow.update(&mut transaction).await?;
    transaction.commit().await?;

    // swap into next step if totp is enabled
    if user.totp_enabled {
        let login_request = UserAuthChallenges::builder()
            .user_id(Some(user.id))
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

    if flow.purpose == AuthChallengePurpose::Signup {
        AuthMailer::new_account(user.login.clone(), user.email.clone(), &global.database).await?;
    }

    AuthMailer::new_session(user.login, user.email, &global.database).await?;

    Ok(RouteEither::Right(Json(AlrightResponse::default())))
}

/// Exchange the flow to enable sudo via a Passkey
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
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if auth.is_authenticated() {
        return Err(ApiErrorCodes::AlreadyAuthenticated);
    }

    let request: PublicKeyCredential = request
        .try_into()
        .map_err(|_| ApiErrorCodes::WebauthnChallengeNotFound)?;

    let Some(challenge_id) = get_challenge_id_from_cookies(&cookies, &global.settings) else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    let Ok(Some(db_challenge)) = UserWebauthnChallenge::take_by_id(
        challenge_id,
        WebauthnChallengeKind::Authenticate,
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    let Ok(Some(user)) = User::find_by_id(db_challenge.user_id, &global.database).await else {
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

    let mut tx = global.database.begin().await?;
    passkey.counter += 1;
    passkey.update(&mut tx).await?;
    tx.commit().await?;

    let session_id = create_session(passkey.user_id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: you opened a new{}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id, &cookies, &global.settings);

    AuthMailer::new_session(user.login, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}

/// Re-Exchange the flow to login via a TOTP code
///
/// This uses the flow id provided by the past exchange method
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

    if !is_flow_correct(&flow) {
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

    let session_id = create_session(user.id, &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;
    create_session_cookie(session_id, &cookies, &global.settings);

    AuthMailer::new_session(user.login, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}
