#![warn(clippy::nursery, clippy::pedantic)]
use std::time::Duration;

use anyhow::Context as _;
use meow_auth2::{
    global::GlobalState,
    http,
    job_queue::QueueRegistry,
    logger,
    mailer::MailerJob,
    manager::Watcher,
    settings::{self, Settings},
};

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
    let watcher = Watcher::new();
    let queues = QueueRegistry::new(global.clone()).register(MailerJob);
    spawn_service("http", http::run(global.clone(), watcher.child()));
    spawn_service("queues", queues.run(watcher.child()));

    let _ = tokio::signal::ctrl_c().await;
    watcher.stop();

    tokio::select! {
        () = watcher.wait() => {tracing::info!("all services stopped gracefully")}
        _ = tokio::signal::ctrl_c() => {tracing::warn!("forcing shutdown")}
        () = kill_timeout(&global.settings) => {tracing::info!("timeout reached, force shutdown")}
    }

    tracing::info!("goodnight");

    Ok(())
}

fn spawn_service<F>(name: &'static str, fut: F)
where
    F: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        let inner = tokio::spawn(fut);
        match inner.await {
            Ok(Ok(())) => tracing::info!("{name} exited normally"),
            Ok(Err(e)) => tracing::error!("{name} exited with an error: {e}"),
            Err(e) => tracing::error!("{name} panicked: {e}"),
        }
    });
}

async fn kill_timeout(settings: &Settings) {
    let timeout = settings
        .application
        .shutdown_timeout_seconds
        .unwrap_or_default();
    if !settings
        .application
        .shutdown_timeout_enabled
        .unwrap_or_default()
        || timeout == 0
    {
        std::future::pending::<()>().await;
    }

    tracing::info!("forcing shutdown in {} seconds", timeout);
    tokio::time::sleep(Duration::from_secs(timeout)).await;
}
