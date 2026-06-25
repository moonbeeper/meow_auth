use std::sync::OnceLock;

use data_encoding::BASE64URL_NOPAD;
use hmac::{Hmac, KeyInit as _, Mac as _};
use sha2::{Digest as _, Sha256};

use crate::{crypto::SecretKey, settings::Settings};

#[derive(Debug)]
pub struct SecretPair {
    pub code: String,
    pub hash: String,
}

static OAUTH_SECRET_KEY: OnceLock<SecretKey> = OnceLock::new();

// god (me, lord space birb), please just add this to the crypto module so I dont have to copy past it two or more times.
type HmacSha256 = Hmac<Sha256>;

fn get_key(settings: &Settings) -> &SecretKey {
    OAUTH_SECRET_KEY.get_or_init(|| {
        settings
            .application
            .master_key
            .derivate("21/06/2026 08:05:01 oauth secrets hmac key", 64)
    })
}

// lord me, please also this
pub fn get_secret_pair(settings: &Settings) -> SecretPair {
    let code = nanoid::nanoid!(128); // LONG ASS STRING :DDDDD = less secure :(

    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(code.as_bytes());
    let result = mac.finalize().into_bytes();
    let hash = hex::encode(result);

    SecretPair { code, hash }
}

// LORD! and also this, me
pub fn verify_secret(secret: &str, hash: &[u8], settings: &Settings) -> bool {
    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(secret.as_bytes());

    mac.verify_slice(hash).is_ok()
}

// could maybe mix it with the get_secret_pair?
pub fn hash_secret(secret: &str, settings: &Settings) -> String {
    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(secret.as_bytes());

    let result = mac.finalize().into_bytes();

    hex::encode(result)
}

pub fn check_pkce(code_verifier: &str, original_code_verifier: &str) -> bool {
    let hashed_challenge = BASE64URL_NOPAD.encode(&Sha256::digest(code_verifier.as_bytes()));

    hashed_challenge == original_code_verifier
}
