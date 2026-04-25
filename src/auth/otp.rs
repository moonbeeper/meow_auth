use argon2::{Argon2, PasswordHasher as _, PasswordVerifier as _};
use rand::{RngExt as _, distr::Alphanumeric};

use crate::database::models::user_auth_challenge::{
    AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges,
};

pub fn generate_otp_code() -> String {
    let code: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    code.to_uppercase()
}

#[derive(Debug)]
pub struct OtpCodePair {
    pub code: String,
    pub hash: String,
}

pub fn get_otp_code() -> anyhow::Result<OtpCodePair> {
    let code = generate_otp_code();
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(code.as_bytes())?.to_string();

    Ok(OtpCodePair { code, hash })
}

pub fn verify_otp_code(code: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();
    let code = code.to_uppercase();

    let Ok(parsed_hash) = argon2::PasswordHash::new(hash) else {
        return false;
    };

    if argon2
        .verify_password(code.as_bytes(), &parsed_hash)
        .is_err()
    {
        return false;
    }

    true
}

pub fn is_flow_correct(flow: &UserAuthChallenges) -> bool {
    if flow.kind != AuthChallengeKind::Otp {
        return false;
    }

    let now = chrono::Utc::now();
    if flow.expires_at < now {
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
