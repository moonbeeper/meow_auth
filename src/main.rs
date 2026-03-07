#![warn(clippy::nursery, clippy::pedantic)]
use anyhow::Context as _;
use meow_auth2::{global::GlobalState, http, logger, settings};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    settings::update_cli();
    let settings = settings::Settings::new().context("Failed to parse settings.")?;
    logger::init(&settings.logging);

    tracing::info!("hi from mr app (creating global state)");
    let global = GlobalState::new(settings)
        .await
        .context("Failed to create global state")?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let http = tokio::spawn(http::run(global, shutdown_rx));

    // all of this below is shit. bad. and bad. It should be replaced with a better way when there's more than one service running.
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown_tx.send(());
    });
    tokio::select! {
        r = http => match r {
            Ok(Ok(())) => tracing::info!("HTTP server exited normally"),
            Ok(Err(e)) => tracing::error!("HTTP server exited with an error: {e}"),
            Err(e) => tracing::error!("HTTP server panicked: {e}")
        },
    }

    tracing::info!("goodnight");

    Ok(())
}
