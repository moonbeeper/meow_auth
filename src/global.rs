use crate::database::PostgresDatabase;
use crate::settings::Settings;
use anyhow::Context;
use sqlx::PgPool;

pub struct GlobalState {
    settings: Settings,
    database: PgPool,
}

impl GlobalState {
    pub async fn new(settings: Settings) -> anyhow::Result<Self> {
        tracing::info!("Creating the global state...");

        let database = PostgresDatabase::connect(&settings.postgres_db)
            .await
            .context("Failed connecting to the postgres database")?;

        tracing::info!("Finalized creating the global state.");
        Ok(Self { settings, database })
    }
}
