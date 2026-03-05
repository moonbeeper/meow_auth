use std::path::Path;

use clap::Parser;
use config::Config;
use smart_default::SmartDefault;
use toml_edit::{Document, DocumentMut};

#[derive(SmartDefault, Debug, serde::Serialize, serde::Deserialize)]
pub struct Logging {
    #[default = true]
    pub enabled: bool,
    pub level: LoggingLevel,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    #[default]
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl std::fmt::Display for LoggingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LoggingLevel::Info => "info",
            LoggingLevel::Warn => "warn",
            LoggingLevel::Error => "error",
            LoggingLevel::Debug => "debug",
            LoggingLevel::Trace => "trace",
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub logging: Logging,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let environment = std::env::var("WORK_ENV").unwrap_or_else(|_| "development".to_string());
        println!("Loading settings for environment '{environment}'");
        Self::create_if_not_exists(&environment)?;
        let settings = Config::builder()
            .add_source(config::File::with_name(&format!(
                "settings/{environment}.toml"
            )))
            .add_source(config::Environment::with_prefix("MEOW_"))
            .build()?;

        match settings.try_deserialize() {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow::anyhow!("Failed to parse settings: {}", e)),
        }
    }

    fn create_if_not_exists(env: &str) -> anyhow::Result<()> {
        let path = Path::new("settings").join(format!("{env}.toml"));

        if !path.exists() {
            std::fs::create_dir_all(Path::new("settings"))?;
            std::fs::write(&path, toml::to_string_pretty(&Self::default())?.as_bytes())?;
            println!("Settings file created at: {}", path.display());
        }

        Ok(())
    }
}

// good enough for now, i mean the thing is going to be always be called on startup of the app lol
#[derive(Debug, clap::Parser)]
struct UpdateCmd {
    /// Environment files to update.
    #[clap(long, short)]
    envs: Vec<String>,
    /// Delete keys that are not present in the default settings.
    #[clap(long, short)]
    delete_unknown_keys: bool,
}

pub fn update_cli() {
    match UpdateCmd::try_parse() {
        Ok(cmd) => {
            if cmd.envs.is_empty() {
                return;
            }

            for env in &cmd.envs {
                Settings::create_if_not_exists(env).expect("Failed creating settings file.");
                let path = Path::new("settings").join(format!("{}.toml", env));
                let contents =
                    std::fs::read_to_string(&path).expect("Failed to read settings file.");
                let current_toml = contents
                    .parse::<Document<String>>()
                    .expect("Failed to parse existing settings file.");

                let mut new_toml = toml::to_string_pretty(&Settings::default())
                    .expect("Failed to get new settings.")
                    .parse::<DocumentMut>()
                    .expect("Failed to parse new settings");

                println!("Starting update for '{env}'");
                update_toml_keys(&mut new_toml, &current_toml, cmd.delete_unknown_keys);
                std::fs::write(path, new_toml.to_string().as_bytes())
                    .expect("Failed to write new settings file.");
                println!("Finished updating settings for environment '{env}'!")
            }
        }
        Err(e) => {
            eprintln!("Something went wrong! {}", e);
            std::process::exit(1)
        }
    }
}

fn update_toml_keys(
    new_toml: &mut DocumentMut,
    current_toml: &Document<String>,
    delete_unknown_keys: bool,
) {
    if delete_unknown_keys {
        println!("Heads up! Going to delete unknown keys in a blink of an eye")
    }
    for (key, value) in current_toml.iter() {
        if !new_toml.contains_key(key) {
            if delete_unknown_keys {
                let name = match value.is_table() {
                    true => "table",
                    false => "key",
                };
                println!("Removing unknown {name} from settings");
                continue;
            }

            if !value.is_table() {
                new_toml.insert(key, value.clone());
                continue;
            }
        }

        if value.is_table()
            && let (Some(new_table), Some(old_table)) = (
                new_toml.get_mut(key).unwrap().as_table_mut(),
                value.as_table(),
            )
        {
            update_toml_table(new_table, old_table, delete_unknown_keys)
        }
    }
}

fn update_toml_table(
    new_table: &mut toml_edit::Table,
    old_table: &toml_edit::Table,
    delete_unknown_keys: bool,
) {
    for (key, value) in old_table.iter() {
        if let Some(new_value) = new_table.get_mut(key) {
            if new_value.is_table() && value.is_table() {
                update_toml_table(
                    new_value.as_table_mut().expect("should be a table"),
                    value.as_table().expect("should be a table"),
                    delete_unknown_keys,
                );
                continue;
            }

            *new_value = value.clone();
        } else {
            if delete_unknown_keys {
                let name = match value.is_table() {
                    true => "table",
                    false => "key",
                };
                println!("Removing unknown {name} '{}' from settings", key);
                continue;
            }

            new_table.insert(key, value.clone());
        }
    }
}
