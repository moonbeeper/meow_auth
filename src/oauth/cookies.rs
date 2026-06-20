use std::sync::OnceLock;

use tower_cookies::{Cookies, cookie};

use crate::{
    auth::{create_cookie, get_cookie},
    database::models::oauth_pending_authorization::OauthPendingAuthorizationId,
    settings::Settings,
};

static OAUTH_KEY: OnceLock<cookie::Key> = OnceLock::new();

fn get_key(settings: &Settings) -> &cookie::Key {
    OAUTH_KEY.get_or_init(|| {
        cookie::Key::from(
            &settings
                .application
                .master_key
                .derivate("20/06/2026 04:12:01 OAUTH cookie key v1", 64),
        )
    })
}

pub fn create_oauth_cookie(
    pending_id: OauthPendingAuthorizationId,
    cookies: &Cookies,
    settings: &Settings,
) {
    create_cookie(
        pending_id.to_string(),
        &format!("{}_pending_oauth", settings.session.cookie_name),
        get_key(settings),
        None,
        cookies,
        settings,
    );
}

pub fn get_oauth_cookie(
    cookies: &Cookies,
    settings: &Settings,
) -> Option<OauthPendingAuthorizationId> {
    let cookie = get_cookie(
        true,
        &format!("{}_pending_oauth", settings.session.cookie_name),
        get_key(settings),
        cookies,
        settings,
    )?;

    match cookie.value().parse::<OauthPendingAuthorizationId>() {
        Ok(v) => Some(v),
        Err(e) => {
            // I frankly don't know if I should have this. maybe i should introduce secrecy around these sites?
            tracing::error!(
                "failed parsing pending oauth authorization cookie value: {}",
                e
            );
            None
        }
    }
}
