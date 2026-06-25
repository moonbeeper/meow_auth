use std::sync::Arc;

use crate::{
    crypto::jwks::create_new_db_jwk, database::models::jwk_key::JwkKey, global::GlobalState,
    job_queue::QueuedJob,
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
