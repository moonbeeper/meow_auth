#![warn(clippy::nursery, clippy::pedantic)]
use std::{io, time::Duration};

use anyhow::Context as _;
use meow_auth2::{
    build,
    crypto::jwks::worker::{JwkCycleWorker, watch_jwk_updates},
    global::GlobalState,
    http,
    job_queue::QueueRegistry,
    logger,
    mailer::MailerJob,
    manager::Watcher,
    settings::{self, Settings},
};
use tokio::signal::unix::{SignalKind, signal};

const BIG_BANNER: &str = r"

                                                                            d8b
                                                                      d8P   ?88
                                                                   d888888P  88b
  88bd8b,d88b  d8888b d8888b  ?88   d8P  d8P     d888b8b  ?88   d8P  ?88'    888888b
  88P'`?8P'?8bd8b_,dPd8P' ?88 d88  d8P' d8P'    d8P' ?88  d88   88   88P     88P `?8b
 d88  d88  88P88b    88b  d88 ?8b ,88b ,88'     88b  ,88b ?8(  d88   88b    d88   88P
d88' d88'  88b`?888P'`?8888P' `?888P'888P'      `?88P'`88b`?88P'?8b  `?8b  d88'   88b
";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{BIG_BANNER}");
    println!("Hello, world!");
    settings::update_cli();
    let settings = settings::Settings::new().context("Failed to parse settings.")?;
    logger::init(&settings.logging);

    tracing::info!(
        "starting meow auth v{} ({})",
        build::PKG_VERSION,
        build::BUILD_TIME
    );
    tracing::info!("hi helooo vroom vroom wawa sqweezeee");

    let global = GlobalState::new(settings)
        .await
        .context("Failed to create global state")?;

    let watcher = Watcher::new();
    let queues = QueueRegistry::new(global.clone())
        .register(MailerJob)
        .register(JwkCycleWorker);

    watcher.spawn_service("http", |child| http::run(global.clone(), child));
    watcher.spawn_service("queues", |child| queues.run(child));
    watcher.spawn_service("update_jwks", |child| {
        watch_jwk_updates(global.clone(), child)
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {tracing::info!("ctrl-c'd! shutting down...")}
        _ = handle_terminate() => {tracing::info!("terminate signal received! shutting down...")}
    }
    watcher.stop();

    tokio::select! {
        () = watcher.wait() => {tracing::info!("all services stopped gracefully")}
        _ = tokio::signal::ctrl_c() => {tracing::warn!("forcing shutdown")}
        () = kill_timeout(&global.settings) => {tracing::info!("timeout reached, force shutdown")}
    }

    tracing::info!("goodnight, sweet bits and flying toasters with wings");

    Ok(())
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

#[allow(clippy::missing_errors_doc)] // stfu
pub async fn handle_terminate() -> io::Result<()> {
    signal(SignalKind::terminate())?.recv().await;
    Ok(())
}
