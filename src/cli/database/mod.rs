use crate::cli::HelpTemplate;
use crate::cli::Run;
use crate::settings::Settings;
use clap::{Parser, Subcommand};
use sqlx::migrate::{AppliedMigration, MigrationType, Migrator};
use std::collections::HashSet;
use std::path::Path;

mod down;
mod reset;
mod status;
mod up;

/// Database related commands
#[derive(Parser, Default)]
#[clap(author, help_template = HelpTemplate, arg_required_else_help(true))]
pub struct Database {
    #[clap(subcommand)]
    pub command: Option<DatabaseCommand>,
}

impl Run for Database {
    async fn run(&self) -> anyhow::Result<()> {
        if let Some(cmd) = &self.command {
            match cmd {
                DatabaseCommand::Down(cmd) => cmd.run().await,
                DatabaseCommand::Up(cmd) => cmd.run().await,
                DatabaseCommand::Reset(cmd) => cmd.run().await,
                DatabaseCommand::Status(cmd) => cmd.run().await,
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Subcommand, Clone)]
pub enum DatabaseCommand {
    Up(up::UpDatabase),
    Down(down::DownDatabase),
    Reset(reset::ResetDatabase),
    Status(status::StatusDatabase),
}

fn hunt_simple_migrations(migrator: &Migrator) {
    let errors: Vec<String> = migrator.iter().filter_map(
        |m| {
            if m.migration_type == MigrationType::Simple {
                Some(format!(
                    "Found simple migration '{} {}'. It cannot be reverted! Use reversible (down / up) migrations instead smh.",
                    m.version,
                    m.description
                ));
            }

            None
        }
    ).collect();

    if errors.is_empty() {
        return;
    }
    println!("{}", errors.join("\n"));
}

fn check_missing_migrations(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    if ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();
    let errors: Vec<String> = applied_migrations
        .iter()
        .filter_map(|m| {
            if !migrations.contains(&m.version) {
                Some(format!("Missing migration: {}", m.version))
            } else {
                None
            }
        })
        .collect();

    if !errors.is_empty() {
        anyhow::bail!(errors.join("\n"));
    }

    Ok(())
}

async fn root_env_file(settings: &Settings) -> anyhow::Result<()> {
    let path = Path::new(".env");

    if !path.exists() {
        tokio::fs::write(&path, []).await?;
    }

    let mut contents = tokio::fs::read_to_string(&path).await?;

    let database_key = contents
        .split("\n")
        .find(|&s| s.starts_with("DATABASE_URL"))
        .map(|s| s.to_string());

    match database_key {
        Some(line) => {
            let parts: Vec<_> = line.splitn(2, "=").collect();
            let value = parts[1];

            if value == settings.postgres_db.uri {
                return Ok(());
            }

            let new_line = format!("DATABASE_URL={}", settings.postgres_db.uri);

            contents = contents.replace(&line, &new_line);
        }
        None => contents.push_str(&format!("\nDATABASE_URL={}", settings.postgres_db.uri)),
    }

    tokio::fs::write(&path, contents.as_bytes()).await?;

    Ok(())
}
