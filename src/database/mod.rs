mod ids;
mod models;

use crate::settings::PostgresDB;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct PostgresDatabase;

impl PostgresDatabase {
    #[allow(clippy::new_ret_no_self)]
    pub async fn connect(settings: &PostgresDB) -> anyhow::Result<PgPool> {
        tracing::info!("Trying to connect to the postgres database...");
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .min_connections(settings.min_connections)
            .connect(&settings.uri)
            .await?;

        tracing::info!("Connected to the postgres database!");
        Ok(pool)
    }
}
