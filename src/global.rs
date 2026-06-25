use std::{sync::Arc, time::Duration};

use anyhow::Context;
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::{crypto::jwks::JwksKeys, database, mailer::Mailer, settings::Settings};

#[derive(Debug)]
pub struct GlobalState {
    pub settings: Settings,
    pub database: sqlx::PgPool,
    pub mailer: Mailer,
    pub webauthn: Arc<Webauthn>,
    pub jwks: JwksKeys,
}

impl GlobalState {
    pub async fn new(settings: Settings) -> anyhow::Result<Arc<Self>> {
        let database = database::setup_pg_database(&settings.database).await?;
        let mailer = Mailer::new(&settings).await?;

        let webauth = WebauthnBuilder::new(&settings.webauthn.rp_id, &settings.http.origin)
            .context("bad webauthn configuration")?;
        let webauth = webauth
            .rp_name(&settings.webauthn.rp_display_name)
            .timeout(Duration::from_secs(
                settings.webauthn.timeout_seconds as u64,
            ))
            .build()?;
        let jwks_list = JwksKeys::new(&database, &settings)
            .await
            .context("failed to get JWKS")?;

        Ok(Arc::new(Self {
            settings,
            database,
            mailer,
            webauthn: Arc::new(webauth),
            jwks: jwks_list,
        }))
    }
}
