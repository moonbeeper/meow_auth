use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};
use webauthn_rs::prelude::{
    CreationChallengeResponse, PasskeyRegistration, RegisterPublicKeyCredential,
};

use crate::{
    auth::webauthn::get_aaguid,
    database::models::{
        user::User,
        user_webauthn::UserWebauthn,
        user_webauthn_challenges::{UserWebauthnChallenge, WebauthnChallengeKind},
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
        v1::types::RegisterPasskeyRequest,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(register_passkey_options))
        .routes(routes!(register_passkey_exchange))
}

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
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    // if auth.is_sudo_enabled() {
    //     return Err(ApiErrorCodes::SudoAlreadyEnabled);
    // }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::Meow);
    };

    // if !user.totp_enabled {
    //     return Err(ApiErrorCodes::SudoAlreadyEnabled);
    // }

    let Ok(passkeys) = UserWebauthn::find_many_by_user_id(user.id, &global.database).await else {
        return Err(ApiErrorCodes::Meow);
    };
    let credential_ids = passkeys
        .iter()
        .map(|v| base64urlsafedata::HumanBinaryData::from(v.credential_id.clone()))
        .collect();

    // exclude credentials are the pid of the already stored passkeys
    let (client_challenge, data) = global
        .webauth
        .start_passkey_registration(
            user.id.into(),
            &user.email,
            &user.login,
            Some(credential_ids),
        )
        .unwrap();

    let data = serde_json::to_value(data).unwrap();
    let db_challenge = UserWebauthnChallenge::builder()
        .user_id(user.id)
        .big_data(data)
        .kind(WebauthnChallengeKind::Register)
        .expires_at(
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.webauthn.timeout_seconds),
        )
        .build();

    let mut tx = global.database.begin().await.unwrap();
    UserWebauthnChallenge::delete_all_by_user(user.id, WebauthnChallengeKind::Register, &mut tx)
        .await
        .unwrap();
    db_challenge.insert(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    Ok(Json(client_challenge))
}

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
) -> Result<(), ApiErrorCodes> {
    let request: RegisterPublicKeyCredential = request
        .try_into()
        .map_err(|_| ApiErrorCodes::TotpInvalidCode)?;
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    // if auth.is_sudo_enabled() {
    //     return Err(ApiErrorCodes::SudoAlreadyEnabled);
    // }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::Meow);
    };

    // if !user.totp_enabled {
    //     return Err(ApiErrorCodes::SudoAlreadyEnabled);
    // }

    let Ok(Some(db_challenge)) = UserWebauthnChallenge::find_by_userid(
        user.id,
        WebauthnChallengeKind::Register,
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::SudoOptionNotAvailable);
    };

    let mut tx = global.database.begin().await.unwrap();
    db_challenge.delete(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    let challenge: PasskeyRegistration = serde_json::from_value(db_challenge.big_data).unwrap();

    let aaguid = get_aaguid(&request.response.attestation_object)
        .ok()
        .map(|v| v.1);

    let passkey = global
        .webauth
        .finish_passkey_registration(&request, &challenge)
        .unwrap();
    let cred_id = passkey.cred_id().clone();

    // assert that the cred id is not already used for another passkey.
    if UserWebauthn::find_by_credential_id(&cred_id, &global.database)
        .await
        .unwrap()
        .is_some()
    {
        return Err(ApiErrorCodes::TotpInvalidCode);
    }

    let big_data = serde_json::to_value(passkey).unwrap();
    let current_passkey_count = UserWebauthn::get_count_by_user_id(user.id, &global.database)
        .await
        .unwrap();

    let passkey = UserWebauthn::builder()
        .user_id(user.id)
        .aaguid(aaguid)
        .credential_id(cred_id.to_vec())
        .display_name(format!("My Passkey {}", current_passkey_count + 1))
        .big_data(big_data)
        .build();

    let mut tx = global.database.begin().await.unwrap();
    passkey.insert(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    Ok(())
}
