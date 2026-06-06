use std::sync::Arc;

use axum::{Extension, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs_proto::RequestChallengeResponse;

use crate::{
    auth::{
        emails::{AuthMailer, EmailVerificationCodeKind},
        otp::get_otp_code,
        webauthn::{create_webauthn_cookie, get_user_passkeys},
    },
    database::models::{
        user::User,
        user_auth_challenge::{AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges},
        user_signup::UserSignup,
        user_webauthn_challenge::{UserWebauthnChallenge, WebauthnChallengeKind},
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::{
            auth::flows::{FlowRequest, FlowResponse},
            types::AuthMethod,
        },
        validator::Valid,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(otp_login))
        .routes(routes!(otp_register))
        .routes(routes!(webauthn_options))
}

/// Start the flow to login via an otp code
#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "login flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_login(
    State(global): State<Arc<GlobalState>>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Err(ApiErrorCodes::AccountNotFound);
    };

    let otp = get_otp_code(&global.settings);

    let login_request = UserAuthChallenges::builder()
        .user_id(Some(user.id))
        .kind(AuthChallengeKind::Otp)
        .secret(Some(otp.hash))
        .build();

    let mut transaction = global.database.begin().await?;
    login_request.insert(&mut transaction).await?;
    transaction.commit().await?;

    AuthMailer::verification_code(
        otp.code,
        EmailVerificationCodeKind::Login,
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

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 4, max = 16))]
    login: String,
    #[validate(email)]
    email: String,
}

/// Start the flow to register an account
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "registration flow created", body = FlowResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_register(
    State(global): State<Arc<GlobalState>>,
    Valid(Json(request)): Valid<Json<RegisterRequest>>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    // awful
    if User::find_by_email(request.email.clone(), &global.database)
        .await?
        .is_some()
    {
        return Err(ApiErrorCodes::EmailAlreadyAssociated);
    }

    if User::find_by_login(request.login.clone(), &global.database)
        .await?
        .is_some()
    {
        return Err(ApiErrorCodes::LoginAlreadyAssociated);
    }

    let otp = get_otp_code(&global.settings);

    let user_signup = UserSignup::builder()
        .email(request.email)
        .login(request.login)
        .build();
    let mut transaction = global.database.begin().await.unwrap();

    let user_signup = user_signup.upsert(&mut transaction).await.unwrap();
    let challenge = UserAuthChallenges::builder()
        .user_signup_id(Some(user_signup.id))
        .kind(AuthChallengeKind::Otp)
        .purpose(AuthChallengePurpose::Signup)
        .secret(Some(otp.hash))
        .build();

    challenge.insert(&mut transaction).await.unwrap();
    transaction.commit().await.unwrap();

    AuthMailer::verification_code(
        otp.code,
        EmailVerificationCodeKind::Register,
        user_signup.login,
        user_signup.email,
        &global.database,
    )
    .await?;

    Ok(Json(FlowResponse {
        flow_id: challenge.id,
        next_method: vec![AuthMethod::Otp],
    }))
}

/// Start the flow to login via a Passkey
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
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<RequestChallengeResponse>, ApiErrorCodes> {
    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Err(ApiErrorCodes::AccountNotFound);
    };

    if !user.has_webauthn {
        return Err(ApiErrorCodes::WebauthnNotEnabled);
    }

    let passkeys = get_user_passkeys(user.id, &global.database)
        .await
        .map_err(|_| ApiErrorCodes::InternalServerError)?;

    let (client_challenge, data) = global.webauthn.start_passkey_authentication(&passkeys)?;

    let data = serde_json::to_value(data)?;
    let db_challenge = UserWebauthnChallenge::builder()
        .user_id(user.id)
        .big_data(data)
        .kind(WebauthnChallengeKind::Authenticate)
        .expires_at(
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.webauthn.timeout_seconds),
        )
        .build();

    let mut tx = global.database.begin().await?;
    UserWebauthnChallenge::delete_all_by_user(
        user.id,
        WebauthnChallengeKind::Authenticate,
        &mut tx,
    )
    .await?;
    db_challenge.insert(&mut tx).await?;
    tx.commit().await?;

    create_webauthn_cookie(db_challenge.id, &cookies, &global.settings);

    Ok(Json(client_challenge))
}
