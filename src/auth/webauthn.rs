use std::sync::OnceLock;

use nom::{bytes::complete::take, error::ParseError, number::complete};
use sqlx::PgPool;
use tower_cookies::{
    Cookies,
    cookie::{self, time},
};
use uuid::Uuid;
use webauthn_rs::prelude::{AuthenticationResult, Passkey};

use crate::{
    database::{
        id::UlidId,
        models::{
            user::UserId, user_webauthn::UserWebauthn,
            user_webauthn_challenges::UserWebauthnChallengeId,
        },
    },
    settings::Settings,
};

static COOKIE_WEBAUTHN_KEY: OnceLock<cookie::Key> = OnceLock::new();

#[derive(serde::Deserialize)]
struct AttestationObject<'a> {
    #[serde(rename = "authData")]
    auth_data: &'a [u8],
}

pub fn get_aaguid(raw_attestation_object: &[u8]) -> nom::IResult<&[u8], Uuid> {
    let parsed: AttestationObject =
        serde_cbor_2::from_slice(raw_attestation_object).map_err(|e| {
            tracing::error!("failed to parse attestation object: {e}");
            nom::Err::Failure(nom::error::Error::from_error_kind(
                raw_attestation_object,
                nom::error::ErrorKind::Fail,
            ))
        })?;

    let (i, _) = take(32usize)(parsed.auth_data)?;
    let (i, flags) = complete::u8(i)?;
    let (i, _) = complete::be_u32(i)?;

    let has_attestation_cred_data = (flags & 0b0100_0000) != 0;
    if !has_attestation_cred_data {
        return Err(nom::Err::Failure(nom::error::Error::from_error_kind(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }

    let (i, aaguid) = take(16usize)(i)?;
    let uuid = Uuid::from_slice(aaguid).map_err(|e| {
        tracing::error!("failed to parse aaguid: {e}");
        nom::Err::Failure(nom::error::Error::from_error_kind(
            i,
            nom::error::ErrorKind::Verify,
        ))
    })?;

    Ok((i, uuid))
}

// TODO: should really mix this and make generic method for both the session and this
pub fn create_webauthn_cookie(challenge_id: UlidId, cookies: &Cookies, settings: &Settings) {
    let encrypted_key = COOKIE_WEBAUTHN_KEY
        .get_or_init(|| cookie::Key::from(settings.webauthn.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    let cookie = cookie::Cookie::build((
        format!("{}_webauthn", settings.session.cookie_name),
        challenge_id.to_string(),
    ))
    .http_only(true)
    .path("/")
    .max_age(time::Duration::seconds(settings.webauthn.timeout_seconds)); // future muaahahah
    cookie_jar.add(cookie.into());
}

fn get_webauthn_cookie(cookies: &Cookies, settings: &Settings) -> Option<cookie::Cookie<'static>> {
    let encrypted_key = COOKIE_WEBAUTHN_KEY
        .get_or_init(|| cookie::Key::from(settings.webauthn.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    let value = cookie_jar.get(&format!("{}_webauthn", settings.session.cookie_name));
    delete_webauthn_cookie(cookies, settings);
    value
}

fn delete_webauthn_cookie(cookies: &Cookies, settings: &Settings) {
    let cookie = cookie::Cookie::build(format!("{}_webauthn", settings.session.cookie_name))
        .http_only(true)
        .path("/");
    cookies.remove(cookie.into());
}

pub fn get_challenge_id_from_cookies(cookies: &Cookies, settings: &Settings) -> Option<UlidId> {
    let cookie = get_webauthn_cookie(cookies, settings)?;
    match cookie.value().parse::<UserWebauthnChallengeId>() {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::error!("failed parsing webauthn cookie value: {}", e);
            None
        }
    }
}

pub async fn get_user_passkeys(user_id: UserId, db: &PgPool) -> anyhow::Result<Vec<Passkey>> {
    let Ok(passkeys) = UserWebauthn::find_many_by_user_id(user_id, db).await else {
        tracing::error!("failed to get user passkeys");
        anyhow::bail!("")
    };

    let passkeys: Vec<_> = passkeys
        .into_iter()
        .flat_map(|v| serde_json::from_value::<Passkey>(v.big_data))
        .collect();

    Ok(passkeys)
}

pub fn update_passkey_with_authentication_result(
    passkey: &mut UserWebauthn,
    auth_result: &AuthenticationResult,
) -> anyhow::Result<()> {
    let mut big_data: Passkey = serde_json::from_value(passkey.big_data.clone())?;
    big_data.update_credential(auth_result);
    passkey.big_data = serde_json::to_value(big_data)?;
    Ok(())
}
