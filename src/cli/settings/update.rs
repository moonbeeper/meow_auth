use crate::cli::Run;
use crate::settings::Settings;
use clap::Parser;
use std::path::Path;
use toml_edit::{Document, DocumentMut, Table};

#[derive(Parser, Clone)]
#[clap(author)]
pub struct UpdateSettings {
    /// Environment/s to update settings for
    #[clap(short, long, default_value = "development")]
    envs: Vec<String>,
    /// Delete unknown keys from existing settings
    #[clap(short, long)]
    delete_unknown: bool,
}

impl Run for UpdateSettings {
    async fn run(&self) -> anyhow::Result<()> {
        if self.envs.is_empty() {
            println!("No work to be done.");
            return Ok(());
        }

        for env in &self.envs {
            let path = Path::new("settings").join(format!("{env}.toml"));
            if !path.exists() {
                println!(
                    "Settings file does not exist for environment '{env}' at path '{path:?}'. Skipping."
                );
                continue;
            }

            let contents = std::fs::read_to_string(&path)?;
            let existing_toml = contents.parse::<Document<String>>()?;
            let mut new_settings =
                toml::to_string_pretty(&Settings::default())?.parse::<DocumentMut>()?;

            println!("Going to update settings for environment '{env}'!");
            update_toml_keys(&mut new_settings, existing_toml, self.delete_unknown);

            std::fs::write(path, new_settings.to_string().as_bytes())?;
            println!("Finished updating settings for environment '{env}'!")
        }

        Ok(())
    }
}

fn update_toml_keys(new: &mut DocumentMut, old: Document<String>, delete_unknown: bool) {
    if delete_unknown {
        println!("Heads up! Deleting unknown keys from existing settings.");
    }

    for (key, value) in old.iter() {
        if !new.contains_key(key) {
            if delete_unknown {
                let name = match value.is_table() {
                    true => "table",
                    false => "key",
                };
                println!("Removing unknown {name} from settings");
                continue;
            }

            if !value.is_table() {
                new.insert(key, value.clone());
                continue;
            }
        }

        if value.is_table()
            && let (Some(new_table), Some(old_table)) =
                (new.get_mut(key).unwrap().as_table_mut(), value.as_table())
        {
            update_toml_table(new_table, old_table, delete_unknown)
        }
    }
}

fn update_toml_table(new: &mut Table, old: &Table, delete_unknown: bool) {
    for (key, value) in old.iter() {
        if let Some(new_value) = new.get_mut(key) {
            if new_value.is_table() && value.is_table() {
                update_toml_table(
                    new_value.as_table_mut().expect("should be a table"),
                    value.as_table().expect("should be a table"),
                    delete_unknown,
                );
                continue;
            }

            *new_value = value.clone();
        } else {
            if delete_unknown {
                let name = match value.is_table() {
                    true => "table",
                    false => "key",
                };
                println!("Removing unknown {name} '{}' from settings", key);
                continue;
            }

            new.insert(key, value.clone());
        }
    }
}
