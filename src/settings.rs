use config::Config;
use smart_default::SmartDefault;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
#[serde(rename_all = "lowercase")]
pub enum LoggingFormat {
    #[default]
    Full,
    Compact,
    Pretty,
    Json,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
pub struct LoggingFile {
    pub enabled: bool,
    #[default = "logs"]
    pub path: PathBuf,
    #[default = 7]
    pub max_count: usize,
    pub format: LoggingFormat,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
pub struct Logging {
    #[default = true]
    pub enabled: bool,
    pub level: LoggingLevel,
    pub file: LoggingFile,
    pub format: LoggingFormat,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
pub struct Settings {
    pub logging: Logging,
}

impl Settings {
    pub fn parse() -> anyhow::Result<Self> {
        let env = std::env::var("MEOW_ENV").unwrap_or_else(|_| "development".into());
        let settings = Config::builder()
            .add_source(config::File::with_name(&format!("settings/{env}.toml")).required(false))
            .add_source(config::File::with_name("settings/local.toml").required(false))
            .build()?;

        match settings.try_deserialize() {
            Ok(settings) => {
                println!("Settings loaded successfully");
                Ok(settings)
            }
            Err(e) => {
                println!("Encountered an error while parsing the settings: {e}");

                if env != "development" {
                    println!("Exiting. (production)");
                    std::process::exit(1);
                }

                println!("Falling back to default settings. (development)");
                Ok(Self::default())
            }
        }
    }

    pub fn generate(name: String, ignore_exists: bool) -> anyhow::Result<()> {
        let path = Path::new("settings").join(format!("{name}.toml"));

        if !ignore_exists && path.exists() {
            anyhow::bail!("File already exists: {path:?}. You might want to update it instead?");
        }

        let mut file = File::create(path)?;
        file.write_all(toml::to_string_pretty(&Self::default())?.as_bytes())?;
        Ok(())
    }
}
