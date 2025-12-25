use config::Config;
use smart_default::SmartDefault;

#[derive(Debug, serde::Deserialize, serde::Serialize, SmartDefault)]
pub struct Settings {
    aaa: String,
}

impl Settings {
    pub fn parse() -> anyhow::Result<Settings> {
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
            Err(_e) => {
                println!("Encountered an error while parsing the settings.");

                if env != "development" {
                    println!("Exiting. (production)");
                    std::process::exit(1);
                }

                println!("Falling back to default settings. (development)");
                Ok(Self::default())
            }
        }
    }
}
