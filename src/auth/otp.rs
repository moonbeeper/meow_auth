use std::sync::OnceLock;

use hmac::{Hmac, KeyInit as _, Mac as _};
use sha2::Sha256;

use crate::{
    crypto::SecretKey,
    database::models::user_auth_challenge::{
        AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges,
    },
    settings::Settings,
};

static OTP_KEY: OnceLock<SecretKey> = OnceLock::new();

const OTP_ALPHABET: [char; 32] = [
    '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L',
    'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct OtpCodePair {
    pub code: String,
    pub hash: String,
}

fn get_otp_key(settings: &Settings) -> &SecretKey {
    OTP_KEY.get_or_init(|| {
        settings
            .application
            .master_key
            .derivate("06/06/2026 23:03:06 otp hmac key", 64)
    })
}

pub fn get_otp_code(settings: &Settings) -> OtpCodePair {
    let code = nanoid::nanoid!(6, &OTP_ALPHABET);

    let mut mac = HmacSha256::new_from_slice(get_otp_key(settings))
        .expect("HMAC key must have an exact length");
    mac.update(code.as_bytes());
    let result = mac.finalize().into_bytes();
    let hash = hex::encode(result);

    OtpCodePair { code, hash }
}

pub fn verify_otp_code(code: &str, hash: &str, settings: &Settings) -> bool {
    let code = code.to_uppercase();

    let mut mac = HmacSha256::new_from_slice(get_otp_key(settings))
        .expect("HMAC key must have an exact length");
    mac.update(code.as_bytes());

    let Ok(hash) = hex::decode(hash) else {
        return false;
    };

    mac.verify_slice(&hash).is_ok()
}

pub fn is_flow_correct(flow: &UserAuthChallenges) -> bool {
    if !super::is_flow_correct(flow, Some(AuthChallengeKind::Otp), None) {
        return false;
    }

    match flow.purpose {
        AuthChallengePurpose::Login => {
            if flow.user_id.is_none() {
                return false;
            }
        }
        AuthChallengePurpose::Signup => {
            if flow.user_signup_id.is_none() {
                return false;
            }
        }
        _ => return false,
    }

    true
}
