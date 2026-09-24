use std::sync::Arc;

use sqlx::postgres::PgListener;

use crate::{
    crypto::jwks::create_new_db_jwk, database::models::jwk_key::JwkKey, global::GlobalState,
    job_queue::QueuedJob, manager::WatcherChild,
};

pub struct JwkCycleWorker;

impl QueuedJob for JwkCycleWorker {
    type Input = ();

    async fn run(&self, global: Arc<GlobalState>, _input: Self::Input) -> anyhow::Result<()> {
        let db_current_jwk = JwkKey::get_active(&global.database).await?;
        let mut tx = global.database.begin().await?;
        let now = chrono::Utc::now();

        match db_current_jwk {
            Some(current_jwk) => {
                if current_jwk.retired_at <= now {
                    let key = create_new_db_jwk(now, &global.settings)?;
                    key.insert(&mut tx).await?;
                }
            }
            None => {
                let key = create_new_db_jwk(now, &global.settings)?;
                key.insert(&mut tx).await?;
            }
        }

        JwkKey::set_retire(&mut tx).await?;
        if JwkKey::delete_non_public(&mut tx).await? {
            // this now only notifies workers when we really really updated something important. Like deleting a key!
            sqlx::query!("notify updated_crypto_jwks")
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        global
            .jwks
            .update(&global.database, &global.settings)
            .await?;

        Self::dispatch_at(
            &global.database,
            chrono::Utc::now()
                + chrono::Duration::seconds(global.settings.oauth.jwk_cycle_after_seconds),
            true,
            (),
        )
        .await?;

        Ok(())
    }
}

/// Watches for notifications from the database when the jwks have been updated
pub async fn watch_jwk_updates(
    global: Arc<GlobalState>,
    shutdown: WatcherChild,
) -> anyhow::Result<()> {
    tracing::info!("watching for jwk update notifications");

    // let mut tick = tokio::time::interval(Duration::from_secs(120));
    let mut listener = PgListener::connect_with(&global.database).await?;
    listener.listen("updated_crypto_jwks").await?;

    loop {
        // this thing exits when the first thing completes.
        tokio::select! {
            _ = shutdown.cancelled() => break,
            // _ = tick.tick() => {}
            _ = listener.recv() => {}
        }

        tracing::info!("received update notification for jwks, updating local keys");
        if let Err(e) = global.jwks.update(&global.database, &global.settings).await {
            tracing::error!("failed to update jwks: {:?}", e)
        }
    }
    Ok(())
}
