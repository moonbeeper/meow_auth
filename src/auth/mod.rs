use std::sync::LazyLock;

use regex::Regex;
use tower_cookies::{
    Cookies,
    cookie::{self, time},
};

use crate::{
    database::models::user_auth_challenge::{
        AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges,
    },
    settings::Settings,
};

pub mod email;
pub mod flags;
pub mod mailer;
pub mod otp;
pub mod session;
pub mod sudo;
pub mod totp;
pub mod webauthn;

pub static RE_AUTH_FLOW_LOGIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

pub fn valid_email(email: &str) -> Result<(), validator::ValidationError> {
    let valid = email_address::EmailAddress::parse_with_options(
        email,
        email_address::Options::default()
            .without_display_text()
            .with_required_tld(),
    );

    if valid.is_err() {
        return Err(validator::ValidationError::new("invalid_email")
            .with_message("must have a valid tld".into()));
    }

    Ok(())
}

/// Basic checks to see if the flow isn't expired and is the correct kind/purpose.
#[allow(
    clippy::collapsible_if,
    reason = "its a hecking if let, not a if statement alone"
)]
pub fn is_flow_correct(
    flow: &UserAuthChallenges,
    kind: Option<AuthChallengeKind>,
    purpose: Option<AuthChallengePurpose>,
) -> bool {
    if let Some(kind) = kind {
        if flow.kind != kind {
            return false;
        }
    }

    if let Some(purpose) = purpose {
        if flow.purpose != purpose {
            return false;
        }
    }

    let now = chrono::Utc::now();
    if flow.expires_at < now {
        return false;
    }
    true
}

pub fn create_cookie(
    data: impl ToString,
    name: &str,
    secret_key: &cookie::Key,
    max_age: Option<i64>,
    cookies: &Cookies,
    settings: &Settings,
) {
    let cookie_jar = cookies.private(secret_key);
    let cookie = cookie::Cookie::build((
        format!("{}_{name}", settings.session.cookie_name),
        data.to_string(),
    ))
    .http_only(true)
    .path("/");

    let cookie = if let Some(max_age) = max_age {
        cookie.max_age(time::Duration::seconds(max_age))
    } else {
        cookie.permanent()
    };

    cookie_jar.add(cookie.into());
}

pub fn get_cookie(
    delete: bool,
    name: &str,
    secret_key: &cookie::Key,
    cookies: &Cookies,
    settings: &Settings,
) -> Option<cookie::Cookie<'static>> {
    let cookie_jar = cookies.private(secret_key);
    let value = cookie_jar.get(&format!("{}_{name}", settings.session.cookie_name));
    if delete {
        delete_cookie(name, cookies, settings);
    }
    value
}

pub fn delete_cookie(name: &str, cookies: &Cookies, settings: &Settings) {
    let cookie = cookie::Cookie::build(format!("{}_{name}", settings.session.cookie_name))
        .http_only(true)
        .path("/");
    cookies.remove(cookie.into());
}
