use crate::cli::HelpTemplate;
use crate::cli::Run;
use clap::{Parser, Subcommand};
mod generate;
mod update;

/// Settings related commands
#[derive(Parser, Default)]
#[clap(author, help_template = HelpTemplate)]
pub struct Settings {
    #[clap(subcommand)]
    pub command: Option<SettingsCommand>,
}

impl Run for Settings {
    async fn run(&self) -> anyhow::Result<()> {
        if let Some(cmd) = &self.command {
            match cmd {
                SettingsCommand::Generate(generate) => generate.run().await,
                SettingsCommand::Update(update) => update.run().await,
            }
        } else {
            println!("No settings command provided. Use --help for more information.");
            Ok(())
        }
    }
}

#[derive(Subcommand, Clone)]
pub enum SettingsCommand {
    Generate(generate::GenerateSettings),
    Update(update::UpdateSettings),
}
