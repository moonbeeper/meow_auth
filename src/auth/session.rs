use std::sync::OnceLock;

use anyhow::Ok;
use sqlx::PgPool;
use tower_cookies::{Cookies, cookie};

use crate::{
    audit::{self, AuditAction},
    auth::{create_cookie, delete_cookie, get_cookie},
    database::{
        id::UlidId,
        models::{
            user::UserId,
            user_session::{PIDUserSessionId, UserSession},
        },
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

    let session = UserSession::builder()
        .user_id(user_id)
        .active_expires_at(active_expires_at)
        .expires_at(expires_at)
        .build();

    let mut tx = db.begin().await?;
    session.insert(&mut tx).await?;
    audit::log(user_id, user_id, AuditAction::SessionCreated, None, &mut tx).await?;
    tx.commit().await?;
    Ok(session.pid)
}

fn get_cookie_key(settings: &Settings) -> &cookie::Key {
    COOKIE_SESSION_KEY.get_or_init(|| {
        cookie::Key::from(
            &settings
                .application
                .master_key
                .derivate("06/06/2026 03:13:39 session cookies v1", 64),
        )
    })
}

pub fn create_session_cookie(session_id: UlidId, cookies: &Cookies, settings: &Settings) {
    create_cookie(
        session_id.to_string(),
        &settings.session.cookie_name,
        get_cookie_key(settings),
        None, // expire can be pushed forward by the renew.
        cookies,
        settings,
    );
}

pub fn get_session_cookie(
    cookies: &Cookies,
    settings: &Settings,
) -> Option<cookie::Cookie<'static>> {
    get_cookie(
        false,
        &settings.session.cookie_name,
        get_cookie_key(settings),
        cookies,
        settings,
    )
}

pub fn delete_session_cookie(cookies: &Cookies, settings: &Settings) {
    delete_cookie(&settings.session.cookie_name, cookies, settings);
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
