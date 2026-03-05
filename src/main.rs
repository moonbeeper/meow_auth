#![warn(clippy::nursery, clippy::pedantic)]
use meow_auth2::{logger, settings};

fn main() {
    settings::update_cli();
    let settings = settings::Settings::new().expect("Failed to parse settings.");
    logger::init(&settings.logging);

    tracing::info!("hi from mr app");
    println!("Hello, world!");
}
