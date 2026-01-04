use crate::cli::database::{check_missing_migrations, hunt_simple_migrations, root_env_file};
use crate::cli::{Run, ask_prompt};
use crate::settings::Settings;
use clap::Parser;
use sqlx::migrate::{Migrate, Migrator};
use sqlx::{Connection, PgConnection};
use std::collections::HashMap;
use std::path::Path;

/// Rollback applied database migrations
#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct DownDatabase {
    /// How many migrations to roll back. If not specified, rolls back one migration
    #[clap(short, long, default_value = "1")]
    count: i64,

    /// Rollback all applied migrations
    #[clap(long)]
    all: bool,

    /// Ignore missing migrations in the migrations folder
    #[clap(long, short)]
    ignore_missing: bool,
}

impl Run for DownDatabase {
    async fn run(&self) -> anyhow::Result<()> {
        let mut count = self.count;
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

        if self.all {
            let result = ask_prompt(format!(
                "Are you sure you want to rollback ALL ({}) applied migrations?",
                applied_migrations.len()
            ))
            .await;

            if !result {
                println!("Aborted the rollback operation.");
                return Ok(());
            }

            count = applied_migrations.len() as i64;
        }

        if count > applied_migrations.len() as i64 {
            anyhow::bail!(
                "Cannot rollback {} migration{}, when only {} have been applied",
                count,
                if count > 1 { "s" } else { "" },
                applied_migrations.len()
            );
        }

        let applied_migrations: HashMap<_, _> = applied_migrations
            .into_iter()
            .map(|m| (m.version, m))
            .collect();

        let mut rolled_back = 0;
        for migration in migrator.iter().rev() {
            if !migration.migration_type.is_down_migration() {
                continue;
            }

            if applied_migrations.contains_key(&migration.version) {
                let elapsed = db_conn.revert(migration).await?;
                println!(
                    "Reverted migration {} {} ({elapsed:?})",
                    migration.version, migration.description
                );
                rolled_back += 1;

                if rolled_back >= count {
                    break;
                }
            }
        }

        println!(
            "Successfully rolled back {} migration{}.",
            rolled_back,
            if rolled_back == 1 { "" } else { "s" }
        );

        let _ = db_conn.close().await;

        Ok(())
    }
}
