use crate::cli::Run;
use crate::cli::database::{check_missing_migrations, hunt_simple_migrations, root_env_file};
use crate::settings::Settings;
use clap::Parser;
use sqlx::migrate::{Migrate, Migrator};
use sqlx::{Connection, PgConnection};
use std::collections::HashMap;
use std::path::Path;

/// Show the status of database migrations
#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct StatusDatabase {
    /// Ignore missing migrations in the migrations folder
    #[clap(long, short)]
    ignore_missing: bool,
}

impl Run for StatusDatabase {
    async fn run(&self) -> anyhow::Result<()> {
        let settings = Settings::parse().expect("Failed to parse settings");
        let migrator = Migrator::new(Path::new("./migrations")).await?;

        let mut db_conn = PgConnection::connect(&settings.postgres_db.uri).await?;
        root_env_file(&settings).await?;
        db_conn.ensure_migrations_table().await?;

        let applied_migrations = db_conn.list_applied_migrations().await?;
        check_missing_migrations(&applied_migrations, &migrator, self.ignore_missing)?;

        if applied_migrations.is_empty() {
            println!("There's no migrations to rollback!");
            return Ok(());
        }

        hunt_simple_migrations(&migrator);

        let applied_migrations: HashMap<_, _> = applied_migrations
            .into_iter()
            .map(|m| (m.version, m))
            .collect();

        for migration in migrator.iter().rev() {
            if migration.migration_type.is_down_migration() {
                continue;
            }

            let applied = applied_migrations.get(&migration.version);
            let status = if let Some(applied) = applied {
                if applied.checksum != migration.checksum {
                    "installed (but modified)"
                } else {
                    "installed"
                }
            } else {
                "not installed"
            };

            println!("{} {} - {status}", migration.version, migration.description);
        }

        let _ = db_conn.close().await;

        Ok(())
    }
}
