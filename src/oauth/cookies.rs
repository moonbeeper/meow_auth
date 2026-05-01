use std::sync::OnceLock;

use tower_cookies::{Cookies, cookie};

use crate::{
    database::models::oauth_pending_authorization::OauthPendingAuthorizationId, settings::Settings,
};

static COOKIE_OAUTH_KEY: OnceLock<cookie::Key> = OnceLock::new();

pub fn create_oauth_cookie(
    authorization_id: OauthPendingAuthorizationId,
    cookies: &Cookies,
    settings: &Settings,
) {
    let encrypted_key =
        COOKIE_OAUTH_KEY.get_or_init(|| cookie::Key::from(settings.oauth.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    let cookie = cookie::Cookie::build((
        format!("{}_pending_oauth", settings.session.cookie_name),
        authorization_id.to_string(),
    ))
    .http_only(true)
    .path("/")
    .permanent(); // future muaahahah
    cookie_jar.add(cookie.into());
}

fn get_oauth_cookie(cookies: &Cookies, settings: &Settings) -> Option<cookie::Cookie<'static>> {
    let encrypted_key =
        COOKIE_OAUTH_KEY.get_or_init(|| cookie::Key::from(settings.oauth.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    let value = cookie_jar.get(&format!("{}_pending_oauth", settings.session.cookie_name));
    delete_oauth_cookie(cookies, settings);
    value
}

fn delete_oauth_cookie(cookies: &Cookies, settings: &Settings) {
    let cookie = cookie::Cookie::build(format!("{}_pending_oauth", settings.session.cookie_name))
        .http_only(true)
        .path("/");
    cookies.remove(cookie.into());
}

pub fn get_pending_authorization_id_from_cookies(
    cookies: &Cookies,
    settings: &Settings,
) -> Option<OauthPendingAuthorizationId> {
    let cookie = get_oauth_cookie(cookies, settings)?;
    match cookie.value().parse::<OauthPendingAuthorizationId>() {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::error!("failed parsing pending authorization cookie value: {}", e);
            None
        }
    }
}
