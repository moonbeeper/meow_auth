use std::sync::Arc;

use axum::{Extension, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::{DiscoverableAuthentication, DiscoverableKey, Passkey};
use webauthn_rs_proto::PublicKeyCredential;

use crate::{
    audit::{AuditAction, AuditEntry},
    auth::{
        mailer::AuthMailer,
        otp::{is_flow_correct, verify_otp_code},
        session::{create_session, create_session_cookie},
        totp::{
            self, TotpCodeState, decrypt_secrets, get_recovery_code_state, get_totp,
            set_recovery_code_used,
        },
        webauthn::get_challenge_id_from_cookies,
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
        middleware::{
            browser_agent_manager::UserAgentContext, ip_manager::IpContext,
            ratelimit_manager::RatelimitLayer,
        },
        v1::{
            auth::flows::FlowResponse,
            types::{AlrightResponse, AuthMethod, AuthenticationPasskeyRequest, RouteEither},
        },
        validator::Valid,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .layer(RatelimitLayer::new(60, chrono::Duration::seconds(60)))
        .routes(routes!(flow_webauthn_exchange))
        .layer(RatelimitLayer::new(20, chrono::Duration::seconds(60)))
        .routes(routes!(flow_otp_exchange))
        .routes(routes!(flow_totp_exchange))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct ExchangeRequest {
    /// The flow ID used to identify and track a authentication flow
    flow_id: UlidId,
    #[validate(length(min = 6, max = 11))]
    code: String,
}

// Workaround for utoipa not liking the RouteEither type. works tho.
#[derive(serde::Serialize, utoipa::ToSchema)]
#[serde(untagged)]
#[allow(unused)]
pub enum _OtpExchangeResponse {
    NeedsTotp(FlowResponse),
    Done(AlrightResponse),
}

/// Exchange the Authentication Flow ID made by an OTP code
///
/// This can be also requested by a user registering a new account.
/// The response can return a `next_method: ["totp"]` if the user has TOTP enabled.
#[utoipa::path(
    post,
    path = "/",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication exchanged successfully", body = _OtpExchangeResponse),
        (status = 401, description = "invalid code", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn flow_otp_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(cookies): Extension<Cookies>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Valid(Json(request)): Valid<Json<ExchangeRequest>>,
) -> Result<RouteEither<Json<FlowResponse>, Json<AlrightResponse>>, ApiErrorCodes> {
    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    if !is_flow_correct(&flow) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let secret_hash = flow
        .secret
        .as_ref()
        .ok_or_else(|| ApiErrorCodes::InvalidCode)?;

    if !verify_otp_code(&request.code, secret_hash, &global.settings) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let user = if flow.purpose == AuthChallengePurpose::Signup {
        let mut tx = global.database.begin().await?;

        let Ok(Some(signup)) = UserSignup::take_by_id(flow.user_signup_id.unwrap(), &mut tx).await
        else {
            return Err(ApiErrorCodes::InvalidCode);
        };

        let user = User::builder()
            .name(signup.email.clone())
            .email(signup.email.clone())
            .email_verified(true)
            .build();

        signup.delete_all_by_email(&mut tx).await?;
        user.insert(&mut tx).await?;

        AuditEntry::builder()
            .user_id(user.id)
            .actor_id(user.id)
            .action(AuditAction::AccountCreated)
            .actor_ip(ip_ctx.ip_addr())
            .actor_location(ip_ctx.location())
            .actor_user_agent(user_agent.agent())
            .build()
            .save(&mut tx)
            .await?;

        flow.state = AuthChallengeState::Completed;
        flow.update(&mut tx).await?;
        tx.commit().await?;

        user
    } else {
        let uid = flow.user_id.ok_or(ApiErrorCodes::InvalidCode)?;
        let Ok(Some(user)) = User::find_by_id(uid, &global.database).await else {
            return Err(ApiErrorCodes::InvalidCode);
        };

        if user.totp_enabled {
            let mut tx = global.database.begin().await?;

            flow.state = AuthChallengeState::Completed;
            flow.update(&mut tx).await?;

            let login_request = UserAuthChallenges::builder()
                .user_id(Some(user.id))
                .kind(AuthChallengeKind::Totp)
                .expires_at(chrono::Utc::now() + chrono::Duration::minutes(10))
                .build();
            login_request.insert(&mut tx).await?;

            tx.commit().await?;

            return Ok(RouteEither::Left(Json(FlowResponse {
                flow_id: login_request.id,
                next_method: vec![AuthMethod::Totp],
            })));
        }

        let mut tx = global.database.begin().await?;
        flow.state = AuthChallengeState::Completed;
        flow.update(&mut tx).await?;
        tx.commit().await?;

        user
    };

    let mut tx = global.database.begin().await?;

    let session_id = create_session(user.id, &mut tx, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;

    AuditEntry::builder()
        .user_id(user.id)
        .actor_id(user.id)
        .action(AuditAction::SessionCreated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;
    create_session_cookie(session_id, &cookies, &global.settings);

    if flow.purpose == AuthChallengePurpose::Signup {
        AuthMailer::new_account(user.name.clone(), user.email.clone(), &global.database).await?;
    }

    AuthMailer::new_session(user.name, user.email, &global.database).await?;

    Ok(RouteEither::Right(Json(AlrightResponse::default())))
}

/// Exchange the Authentication Flow made by a Passkey browser challenge
#[utoipa::path(
    post,
    path = "/webauthn",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication exchanged successfully", body = AlrightResponse),
        (status = 404, description = "webauthn challenge not found", body = ApiError),
        (status = 403, description = "webauthn compromised", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn flow_webauthn_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(cookies): Extension<Cookies>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Json(request): Json<AuthenticationPasskeyRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
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

    let Ok(Some(mut db_passkey)) =
        UserWebauthn::find_by_credential_id(request.get_credential_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    if !db_passkey.enabled {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    }

    let mut passkey: Passkey = serde_json::from_value(db_passkey.big_data.clone())?;

    let challenge: DiscoverableAuthentication = serde_json::from_value(db_challenge.big_data)?;

    let auth_result = global.webauthn.finish_discoverable_authentication(
        &request,
        challenge,
        &[DiscoverableKey::from(&passkey)],
    )?;

    let Ok(Some(user)) = User::find_by_id(db_passkey.user_id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    // WARNING: 1Password synced passkeys, have a counter of 0 always.
    // while hardware should ahve normal counters.

    // check counter to account for cloning attackssss
    // TODO: move to helper method for common usage in login and sudo
    // past me wtf, "less and equal" IS NOT THE RIGHT THING. dumb bird brain, a new passkey has a counter of 0!
    if auth_result.counter() < db_passkey.counter as u32 {
        let mut tx = global.database.begin().await?;
        db_passkey.enabled = false;
        db_passkey.update(&mut tx).await?;
        tx.commit().await?;
        AuthMailer::webauthn_compromised(
            user.name,
            db_passkey.display_name,
            user.email,
            &global.database,
        )
        .await?;
        return Err(ApiErrorCodes::WebauthnCompromised);
    }

    // update passkey
    passkey.update_credential(&auth_result);
    db_passkey.big_data = serde_json::to_value(passkey)?;
    db_passkey.counter = auth_result.counter().cast_signed();

    let mut tx = global.database.begin().await?;
    db_passkey.update(&mut tx).await?;

    let session_id = create_session(user.id, &mut tx, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;

    AuditEntry::builder()
        .user_id(user.id)
        .actor_id(user.id)
        .action(AuditAction::SessionCreated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    create_session_cookie(session_id, &cookies, &global.settings);

    AuthMailer::new_session(user.name, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}

/// Exchange the Authentication Flow ID made by an enabled TOTP
#[utoipa::path(
    post,
    path = "/totp",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication exchanged successfully", body = AlrightResponse),
        (status = 401, description = "invalid code", body = ApiError),
        (status = 400, description = "totp recovery code already used", body = ApiError),
        (status = 404, description = "authentication flow not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn flow_totp_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(cookies): Extension<Cookies>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Valid(Json(request)): Valid<Json<ExchangeRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut flow)) =
        UserAuthChallenges::find_by_id(request.flow_id, &global.database).await
    else {
        // non oracle move BECAUSE USERS, LIKE me, get mad trying to think about if something i touched has broken
        // the whole chain of flow and bleh bleh blah WAWAW I AM MAD THIS SPACEBIRD I MAD OKAY!?
        return Err(ApiErrorCodes::FlowNotFound);
    };

    if !crate::auth::is_flow_correct(&flow, Some(AuthChallengeKind::Totp), None) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    // short circuit if the user doesn't exist
    let Ok(Some(user)) = User::find_by_id(flow.user_id.unwrap(), &global.database).await else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    // short circuit if the user doesn't have totp enabled.
    // this can get people confused (like mr right now) if it was a internal server error.
    if !user.totp_enabled {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let Ok(Some(mut db_totp)) = UserTotp::find_one_by_user(user.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;

    if request.code.len() == 6 {
        let totp_client =
            get_totp(user.name.clone(), totp.secret, &global.settings).map_err(|e| {
                tracing::error!("something went wrong while creating the totp client: {e}");
                ApiErrorCodes::InternalServerError
            })?;

        if totp::check_current(&request.code, totp_client, &mut db_totp, &global.database)
            .await
            .is_err()
        {
            return Err(ApiErrorCodes::InvalidCode);
        }
    } else {
        let state = get_recovery_code_state(&db_totp, &totp.recovery_secret, request.code.clone());
        if let TotpCodeState::Unused(idx) = state {
            let mut tx = global.database.begin().await?;
            set_recovery_code_used(idx, &mut db_totp, &mut tx)
                .await
                .map_err(|e| {
                    tracing::error!(
                        "something went wrong while setting the used totp recovery code: {e}"
                    );
                    ApiErrorCodes::InternalServerError
                })?;

            AuditEntry::builder()
                .user_id(user.id)
                .actor_id(user.id)
                .action(AuditAction::TotpRecoveryCodeUsed)
                .actor_ip(ip_ctx.ip_addr())
                .actor_location(ip_ctx.location())
                .actor_user_agent(user_agent.agent())
                .build()
                .save(&mut tx)
                .await?;

            tx.commit().await?;
            AuthMailer::totp_recovery_code_used(
                user.name.clone(),
                user.email.clone(),
                &global.database,
            )
            .await?;
        } else {
            return Err(ApiErrorCodes::TotpRecoveryAlreadyUsed);
        }
    }

    let mut tx = global.database.begin().await?;
    flow.state = AuthChallengeState::Completed;
    flow.update(&mut tx).await?;

    let session_id = create_session(user.id, &mut tx, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("failed creating session: {}", e);
            ApiErrorCodes::InternalServerError
        })?;

    AuditEntry::builder()
        .user_id(user.id)
        .actor_id(user.id)
        .action(AuditAction::SessionCreated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;
    create_session_cookie(session_id, &cookies, &global.settings);

    AuthMailer::new_session(user.name, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}
