pub mod error;
pub mod id;
pub mod models;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::settings::DatabaseSettings;

pub async fn setup_pg_database(settings: &DatabaseSettings) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .min_connections(settings.min_connections)
        .connect(&settings.url)
        .await?;
    Ok(pool)
}
