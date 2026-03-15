use std::sync::OnceLock;

use anyhow::Ok;
use sqlx::PgPool;
use tower_cookies::{Cookies, cookie};

use crate::{
    database::models::{
        user::UserId,
        user_session::{PIDUserSessionId, UserSession},
    },
    settings::Settings,
};

static COOKIE_SESSION_KEY: OnceLock<cookie::Key> = OnceLock::new();

pub async fn create_session(
    user_id: UserId,
    db: &PgPool,
    settings: &Settings,
) -> anyhow::Result<PIDUserSessionId> {
    let expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(settings.session.expire_age_seconds);
    let active_expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(settings.session.active_expire_age_seconds);

    let mut transaction = db.begin().await?;
    let session = UserSession::builder()
        .user_id(user_id)
        .active_expires_at(active_expires_at)
        .expires_at(expires_at)
        .build();
    session.insert(&mut transaction).await?;
    transaction.commit().await?;
    Ok(session.pid)
}

// TODO: should really use the session model to create the cookie to be able to set the expire time on the cookie
pub fn create_session_cookie(session_id: String, cookies: &Cookies, settings: &Settings) {
    let encrypted_key = COOKIE_SESSION_KEY
        .get_or_init(|| cookie::Key::from(settings.session.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    let cookie = cookie::Cookie::build((settings.session.cookie_name.clone(), session_id))
        .http_only(true)
        .permanent(); // future muaahahah
    cookie_jar.add(cookie.into());
}

pub fn get_session_cookie(
    cookies: &Cookies,
    settings: &Settings,
) -> Option<cookie::Cookie<'static>> {
    let encrypted_key = COOKIE_SESSION_KEY
        .get_or_init(|| cookie::Key::from(settings.session.secret_key.as_bytes()));
    let cookie_jar = cookies.private(encrypted_key);
    cookie_jar.get(&settings.session.cookie_name)
}

pub fn delete_session_cookie(cookies: &Cookies, settings: &Settings) {
    let cookie = cookie::Cookie::build(settings.session.cookie_name.clone()).http_only(true);
    cookies.remove(cookie.into());
}

pub async fn renew_session(
    session: &mut UserSession,
    db: &PgPool,
    settings: &Settings,
) -> anyhow::Result<()> {
    let active_expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(settings.session.active_expire_age_seconds);

    let mut transaction = db.begin().await?;
    session.active_expires_at = active_expires_at;
    session.update(&mut transaction).await?;
    transaction.commit().await?;
    Ok(())
}
