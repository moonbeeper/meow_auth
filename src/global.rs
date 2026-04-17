use std::{sync::Arc, time::Duration};

use anyhow::Context;
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::{database, mailer::Mailer, settings::Settings};

#[derive(Debug)]
pub struct GlobalState {
    pub settings: Settings,
    pub database: sqlx::PgPool,
    pub mailer: Mailer,
    pub webauth: Arc<Webauthn>,
}

impl GlobalState {
    pub async fn new(settings: Settings) -> anyhow::Result<Arc<Self>> {
        let database = database::setup_pg_database(&settings.database).await?;
        let mailer = Mailer::new(&settings).await?;

        let webauth = WebauthnBuilder::new(&settings.webauthn.rp_id, &settings.http.origin)
            .context("bad webauthn configuration")?;
        let webauth = webauth
            .rp_name(&settings.webauthn.rp_name)
            .timeout(Duration::from_secs(
                settings.webauthn.timeout_seconds as u64,
            ))
            .build()?;

        Ok(Arc::new(Self {
            settings,
            database,
            mailer,
            webauth: Arc::new(webauth),
        }))
    }
}
