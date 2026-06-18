use std::sync::OnceLock;

use hmac::{Hmac, KeyInit as _, Mac as _};
use sha2::Sha256;

use crate::{crypto::SecretKey, settings::Settings};

type HmacSha256 = Hmac<Sha256>;

static EMAIL_KEY: OnceLock<SecretKey> = OnceLock::new();

// TODO: should make generic method in the crypto module.
fn get_key(settings: &Settings) -> &SecretKey {
    EMAIL_KEY.get_or_init(|| {
        settings.application.master_key.derivate(
            "18/06/2026 04:21:52 email modification verification hmac key",
            64,
        )
    })
}

// TODO: should merge with OTP CodePair.
// TODO: should make generic "get_token" and "verify_token" method that takes a key and returns a codepair in the crypto module.
#[derive(Debug)]
pub struct TokenPair {
    pub token: String,
    pub hash: Vec<u8>,
}

pub fn get_token(settings: &Settings) -> TokenPair {
    let token = nanoid::nanoid!(32);

    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(token.as_bytes());
    let result = mac.finalize().into_bytes();

    TokenPair {
        token,
        hash: result.to_vec(),
    }
}

pub fn hash_token(token: &str, settings: &Settings) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(token.as_bytes());
    let hash = mac.finalize().into_bytes();

    hash.to_vec()
}

pub fn verify_token(token: &str, hash: &str, settings: &Settings) -> bool {
    let token = token.to_uppercase();

    let mut mac =
        HmacSha256::new_from_slice(get_key(settings)).expect("HMAC key must have an exact length");
    mac.update(token.as_bytes());

    let Ok(hash) = hex::decode(hash) else {
        return false;
    };

    mac.verify_slice(&hash).is_ok()
}
