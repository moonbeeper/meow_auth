use std::sync::Arc;

use crate::{database, mailer::Mailer, settings::Settings};

pub struct GlobalState {
    pub settings: Settings,
    pub database: sqlx::PgPool,
    pub mailer: Mailer,
}

impl GlobalState {
    pub async fn new(settings: Settings) -> anyhow::Result<Arc<Self>> {
        let database = database::setup_pg_database(&settings.database).await?;
        let mailer = Mailer::new(&settings).await?;

        Ok(Arc::new(Self {
            settings,
            database,
            mailer,
        }))
    }
}
