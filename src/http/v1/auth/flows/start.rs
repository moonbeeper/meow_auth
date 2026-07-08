use std::sync::Arc;

use axum::{Extension, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs_proto::RequestChallengeResponse;

use crate::{
    auth::{
        mailer::{AuthMailer, EmailVerificationCodeKind},
        otp::get_otp_code,
        webauthn::{create_webauthn_cookie, get_user_passkeys},
    },
    database::{
        id::UlidId,
        models::{
            user::User,
            user_auth_challenge::{AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges},
            user_signup::UserSignup,
            user_webauthn_challenge::{UserWebauthnChallenge, WebauthnChallengeKind},
        },
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

/// Authenticate via an OTP code
///
/// Starts the flow to authenticate via an OTP sent to the user's email
#[utoipa::path(
    post,
    path = "/",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication flow created", body = FlowResponse),
        // (status = 404, description = "account not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_login(
    State(global): State<Arc<GlobalState>>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        // return Err(ApiErrorCodes::AccountNotFound); HECK YOU >:( it will always say "invalid code" great job me! really great job!
        return Ok(Json(FlowResponse {
            flow_id: UlidId::new(),
            next_method: vec![AuthMethod::Otp],
        }));
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
        user.name,
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
    // #[validate( // mr fmt doesnt format this aberration.
    //     length(min = 3, max = 63, message = "must be between 4 letters and 64"), // counts from 0 duh
    //     regex(path = *RE_AUTH_FLOW_LOGIN, message = "must be alphanumeric and can contain underscores")
    // )]
    // login: String,
    #[validate(custom(function = "crate::auth::valid_email"))]
    email: String,
}

/// Register a new account via an OTP code
///
/// Starts the flow to register a new account via an OTP sent to the user's email.
/// Used on the same exchange endpoint as the authentication OTP code flow.
#[utoipa::path(
    post,
    path = "/register",
    tags = ["auth"],
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "registration flow created", body = FlowResponse),
        // (status = 400, description = "email or login already associated", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn otp_register(
    State(global): State<Arc<GlobalState>>,
    Valid(Json(request)): Valid<Json<RegisterRequest>>,
) -> Result<Json<FlowResponse>, ApiErrorCodes> {
    // awful
    if let Some(user) = User::find_by_email(request.email.clone(), &global.database).await? {
        // return Err(ApiErrorCodes::EmailAlreadyAssociated);
        // DIRTY JOB! TODO: cleanup and separate the otp login into another method to be able to reuse it
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
            user.name,
            user.email,
            &global.database,
        )
        .await?;

        return Ok(Json(FlowResponse {
            flow_id: login_request.id,
            next_method: vec![AuthMethod::Otp],
        }));
    }

    let otp = get_otp_code(&global.settings);

    // I somehow forgot about this having unwraps lol
    let user_signup = UserSignup::builder().email(request.email).build();
    let mut transaction = global.database.begin().await?;

    let user_signup = user_signup.upsert(&mut transaction).await?;
    let challenge = UserAuthChallenges::builder()
        .user_signup_id(Some(user_signup.id))
        .kind(AuthChallengeKind::Otp)
        .purpose(AuthChallengePurpose::Signup)
        .secret(Some(otp.hash))
        .build();

    challenge.insert(&mut transaction).await?;
    transaction.commit().await?;

    AuthMailer::verification_code(
        otp.code,
        EmailVerificationCodeKind::Register,
        user_signup.email.clone(),
        user_signup.email,
        &global.database,
    )
    .await?;

    Ok(Json(FlowResponse {
        flow_id: challenge.id,
        next_method: vec![AuthMethod::Otp],
    }))
}

/// Authenticate via a Passkey
///
/// Returns the challenge for the user's browser to use to authenticate.
#[utoipa::path(
    post,
    path = "/webauthn",
    tags = ["auth"],
    responses(
        (status = 200, description = "authentication flow created"),
        // (status = 404, description = "account not found", body = ApiError),
        (status = 400, description = "user has webauthn not enabled", body = ApiError), // should i be even returning 400 here?
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn webauthn_options(
    State(global): State<Arc<GlobalState>>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<FlowRequest>,
) -> Result<Json<RequestChallengeResponse>, ApiErrorCodes> {
    let Ok(Some(user)) = User::find_by_email(request.email, &global.database).await else {
        return Err(ApiErrorCodes::WebauthnNotEnabled); // mask the existence of the user
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
