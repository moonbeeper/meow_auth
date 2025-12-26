use crate::cli::Run;
use crate::settings::Settings;
use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[clap(author)]
pub struct GenerateSettings {
    /// Environment/s to generate settings for
    #[clap(short, long, default_value = "development")]
    envs: Vec<String>,
    /// Overwrite the settings file if it already exists
    #[clap(short, long)]
    overwrite: bool,
}

impl Run for GenerateSettings {
    async fn run(&self) -> anyhow::Result<()> {
        if self.envs.is_empty() {
            println!("No work to be done.");
            return Ok(());
        }

        for env in &self.envs {
            match Settings::generate(env.clone(), self.overwrite) {
                Ok(_) => {
                    println!("Settings generated at 'settings/{env}.toml'");
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
