use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::{
    CreationChallengeResponse, PasskeyRegistration, RegisterPublicKeyCredential,
};

use crate::{
    auth::{emails::AuthMailer, webauthn::get_aaguid},
    database::{
        id::UlidId,
        models::{
            user::User,
            user_webauthn::UserWebauthn,
            user_webauthn_challenge::{UserWebauthnChallenge, WebauthnChallengeKind},
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{auth_manager::AuthContext, require_auth::RequireAuthenticationLayer},
        v1::types::{AlrightResponse, Passkey, RegisterPasskeyRequest},
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(register_passkey_options))
        .routes(routes!(register_passkey_exchange))
        .routes(routes!(list_passkeys))
        .routes(routes!(delete_passkey))
        .layer(RequireAuthenticationLayer::new())
}

/// Get the passkey creation options
#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "passkey creation challenge options"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn register_passkey_options(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<CreationChallengeResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let Ok(passkeys) = UserWebauthn::find_many_by_user_id(user.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let credential_ids = passkeys
        .iter()
        .map(|v| base64urlsafedata::HumanBinaryData::from(v.credential_id.clone()))
        .collect();

    // exclude credentials are the pid of the already stored passkeys
    let (client_challenge, data) = global.webauthn.start_passkey_registration(
        user.id.into(),
        &user.email,
        &user.login,
        Some(credential_ids),
    )?;

    let data = serde_json::to_value(data)?;
    let db_challenge = UserWebauthnChallenge::builder()
        .user_id(user.id)
        .big_data(data)
        .kind(WebauthnChallengeKind::Register)
        .expires_at(
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.webauthn.timeout_seconds),
        )
        .build();

    let mut tx = global.database.begin().await?;
    UserWebauthnChallenge::delete_all_by_user(user.id, WebauthnChallengeKind::Register, &mut tx)
        .await?;
    db_challenge.insert(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(client_challenge))
}

/// Exchange the passkey register result created on the browser
#[utoipa::path(
    post,
    path = "/exchange",
    responses(
        (status = 200, description = "passkey creation successful"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn register_passkey_exchange(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<RegisterPasskeyRequest>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let request: RegisterPublicKeyCredential = request
        .try_into()
        .map_err(|_| ApiErrorCodes::WebauthnChallengeNotFound)?;

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let Ok(Some(db_challenge)) = UserWebauthnChallenge::take_by_user_id(
        user.id,
        WebauthnChallengeKind::Register,
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    };

    let db_challenge: PasskeyRegistration = serde_json::from_value(db_challenge.big_data)?;
    let aaguid = get_aaguid(&request.response.attestation_object)
        .ok()
        .map(|v| v.1);

    let passkey = global
        .webauthn
        .finish_passkey_registration(&request, &db_challenge)?;
    let cred_id = passkey.cred_id().clone();

    // assert that the cred id is not already used for another passkey.
    if UserWebauthn::find_by_credential_id(&cred_id, &global.database)
        .await?
        .is_some()
    {
        return Err(ApiErrorCodes::WebauthnChallengeNotFound);
    }

    let big_data = serde_json::to_value(passkey)?;
    let current_passkey_count =
        UserWebauthn::get_count_by_user_id(user.id, &global.database).await?;

    let passkey_name = format!("My Passkey {}", current_passkey_count + 1);

    let passkey = UserWebauthn::builder()
        .user_id(user.id)
        .aaguid(aaguid)
        .credential_id(cred_id.to_vec())
        .display_name(passkey_name.clone())
        .big_data(big_data)
        .build();

    let mut tx = global.database.begin().await?;
    user.has_webauthn = true;
    user.update(&mut tx).await?;
    passkey.insert(&mut tx).await?;
    tx.commit().await?;

    AuthMailer::webauthn_registered(user.login, passkey_name, user.email, &global.database).await?;

    Ok(Json(AlrightResponse::default()))
}

/// List all your created passkeys
#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "a list of passkeys", body = Vec<Passkey>),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn list_passkeys(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<Vec<Passkey>>, ApiErrorCodes> {
    let Ok(passkeys) = UserWebauthn::find_many_by_user_id(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let passkeys: Vec<_> = passkeys.into_iter().map(Passkey::from).collect();
    Ok(Json(passkeys))
}

#[derive(Debug, serde::Deserialize)]
pub struct PasskeyQuery {
    id: UlidId,
}

/// Delete one of your passkeys
///
/// You use the ID of one of your passkeys.
#[utoipa::path(
    delete,
    path = "/{id}",
    params(
        ("id" = UlidId, description = "the id of the passkey to delete")
    ),
    responses(
        (status = 200, description = "successfully deleted the passkey"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn delete_passkey(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(query): Path<PasskeyQuery>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    if !auth.is_sudo_enabled() {
        return Err(ApiErrorCodes::SudoNotEnabled);
    }

    let Ok(Some(session)) = UserWebauthn::find_by_pid(query.id, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    if session.user_id != auth.user_id() {
        return Err(ApiErrorCodes::WebauthnNotFound);
    }

    let mut tx = global.database.begin().await?;
    session.delete(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
