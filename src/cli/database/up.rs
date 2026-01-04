use crate::cli::Run;
use crate::cli::database::{check_missing_migrations, hunt_simple_migrations, root_env_file};
use crate::settings::Settings;
use clap::Parser;
use sqlx::migrate::{Migrate, Migrator};
use sqlx::{Connection, PgConnection};
use std::collections::HashMap;
use std::path::Path;

/// Apply all pending database migrations
#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct UpDatabase {
    /// Ignore missing migrations in the migrations folder
    #[clap(long, short)]
    pub(crate) ignore_missing: bool,
}

impl Run for UpDatabase {
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

            match applied_migrations.get(&migration.version) {
                Some(applied_migration) => {
                    if migration.checksum != applied_migration.checksum {
                        anyhow::bail!(
                            "You changed the migration '{} {}' after it was applied... didn't you?",
                            migration.version,
                            migration.description
                        );
                    }
                }
                None => {
                    let elapsed = db_conn.apply(migration).await?;
                    println!(
                        "Applied migration {} {} ({elapsed:?})",
                        migration.version, migration.description
                    );
                }
            }
        }

        let _ = db_conn.close().await;

        println!("Finished applying migrations.");

        Ok(())
    }
}
