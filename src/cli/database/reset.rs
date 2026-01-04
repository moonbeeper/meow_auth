use crate::cli::database::root_env_file;
use crate::cli::{Run, ask_prompt};
use crate::settings::Settings;
use clap::Parser;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Any, Connection, PgConnection};

/// Reset the database by dropping and recreating it
#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct ResetDatabase {
    /// Force the reset of the database
    #[clap(short, long)]
    force: bool,

    /// Recreate the database after dropping it
    #[clap(short = 'c', long, default_value = "true")]
    recreate: bool,
}

impl Run for ResetDatabase {
    async fn run(&self) -> anyhow::Result<()> {
        let settings = Settings::parse()?;
        let db_conn = PgConnection::connect(&settings.postgres_db.uri).await;

        if db_conn.is_err() {
            println!("How about having the database reachable first? Aborted.");
            return Ok(());
        }

        root_env_file(&settings).await?;

        let decision = ask_prompt(format!(
            "Are you really {}sure you want to {} reset the database?",
            if self.force { "REALLY " } else { "" },
            if self.force { "FORCE" } else { "" }
        ))
        .await;

        if !decision {
            println!(
                "Aborted the database {} reset operation.",
                if self.force { "force" } else { "" }
            )
        }

        if self.force {
            Any::force_drop_database(&settings.postgres_db.uri).await?;
        }

        Any::drop_database(&settings.postgres_db.uri).await?;

        let _ = db_conn?.close().await;

        println!("Database dropped");

        if self.recreate {
            println!("Recreating database...");
            super::up::UpDatabase {
                ignore_missing: false,
            }
            .run()
            .await?;
        }

        Ok(())
    }
}
