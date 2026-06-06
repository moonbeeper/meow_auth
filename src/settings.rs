use std::{net::SocketAddr, path::Path};

use crate::crypto::{SecretKey, get_secret_key};
use anyhow::Context;
use clap::Parser;
use config::Config;
use smart_default::SmartDefault;
use toml_edit::{Document, DocumentMut};
use url::Url;

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

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[default(Some(60*2))]
    pub shutdown_timeout_seconds: Option<u64>,
    #[default(Some(true))]
    pub shutdown_timeout_enabled: Option<bool>,
    #[default(get_secret_key(32))]
    pub master_key: SecretKey,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct HttpSettings {
    #[default(SocketAddr::from(([127,0,0,1],8080)))]
    pub bind: SocketAddr,
    pub api_docs: ApiDocsSettings,
    #[default(Url::parse("http://localhost:8080").expect("failed to parse default origin burh"))]
    pub origin: Url,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct ApiDocsSettings {
    #[default = true]
    pub enabled: bool,
    #[default("/scalar".to_string())]
    pub path: String,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    #[default("postgres://meow:meow_pwd@localhost:5432/meow2_development".to_string())]
    pub url: String,
    #[default = 1]
    pub min_connections: u32,
    #[default = 10]
    pub max_connections: u32,
}

// TODO: Should find a better name for this.
#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct SessionSettings {
    #[default("meow_sess")]
    pub cookie_name: String,
    #[default(60 * 60 * 24 * 30)]
    pub expire_age_seconds: i64,
    #[default(60 * 60 * 24 * 7)]
    pub active_expire_age_seconds: i64,
    #[default(60 * 10)]
    pub update_threshold_seconds: i64,
    #[default(60 * 15)]
    pub sudo_expire_age_seconds: i64,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct MailerSettings {
    #[default("localhost".to_string())]
    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    #[default = 1025]
    pub smtp_port: u16,
    #[default = false]
    pub smtp_secure: bool,
    #[default("meow@meow.com".to_string())]
    pub from_email: String,
    #[default = false]
    pub test_connection: bool,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct TotpSettings {
    #[default("meow_auth".to_string())]
    pub issuer: String,
    #[default(6)]
    pub digits: usize,
}

#[derive(Debug, SmartDefault, serde::Serialize, serde::Deserialize, Clone)]
pub struct WebauthnSettings {
    #[default("localhost".to_string())]
    pub rp_id: String,
    #[default("MeowAuth".to_string())]
    pub rp_name: String,
    #[default(60*5)]
    pub timeout_seconds: i64,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub application: ApplicationSettings,
    pub http: HttpSettings,
    pub database: DatabaseSettings,
    pub logging: Logging,
    pub mailer: MailerSettings,
    pub session: SessionSettings,
    pub totp: TotpSettings,
    pub webauthn: WebauthnSettings,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().context("Failed to load .env* file")?;
        let environment = std::env::var("WORK_ENV").unwrap_or_else(|_| "development".to_string());
        println!("Loading settings for environment '{environment}'");
        // Self::create_if_not_exists(&environment)?; shouldn't be auto creating the file.
        let settings = Config::builder()
            .add_source(
                config::File::with_name(&format!("settings/{environment}.toml")).required(false),
            )
            .add_source(config::Environment::with_prefix("MEOW").separator("__"))
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
