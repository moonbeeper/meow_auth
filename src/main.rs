#![warn(clippy::nursery, clippy::pedantic)]

use meow_auth::{global, http, logging, settings};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let settings = settings::Settings::parse().expect("Failed trying to load settings.");
    logging::init(&settings.logging);
    let global = Arc::new(global::GlobalState::new(settings));

    let shutdown_channel = tokio::sync::oneshot::channel::<()>();
    let http_srv = tokio::spawn(http::run(global, shutdown_channel.1));

    let shutdown = tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        println!("Shutting down...");
        shutdown_channel.0.send(()).unwrap();

        tokio::time::timeout(std::time::Duration::from_secs(60), tokio::signal::ctrl_c())
            .await
            .ok()
    });

    tokio::select! {
        r = http_srv => match r {
            Ok(Ok(())) => println!("HTTP server exited normally."),
            Ok(_) => println!("HTTP server exited."),
            Err(e) => println!("HTTP server forcefully exited because of: {e}")
        },
        _ = shutdown => {
            println!("Shutdown now.");
        }
    }
    // build our application with a single route
}
