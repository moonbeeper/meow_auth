pub mod worker;

use std::sync::{Arc, OnceLock};

use anyhow::Context;
use arc_swap::ArcSwap;
use chrono::Utc;
use compact_jwt::{JwsEs256Signer, JwsSigner};
use sqlx::PgPool;

use crate::{
    crypto::{EncryptedSecret, SecretKey, decrypt_secret, encrypt_secret},
    database::models::jwk_key::{JwkKey as DBJwkKey, JwkKeyId},
    job_queue::QueuedJob,
    settings::Settings,
};

static JWKS_KEY: OnceLock<SecretKey> = OnceLock::new(); // should always be set

#[derive(Debug, Clone)]
pub struct JwkKey {
    pub id: JwkKeyId,
    pub signer: JwsEs256Signer,
}

#[derive(Debug)]
pub struct JwksKeysInner {
    // contains retired and active public keys
    pub keys: Vec<JwkKey>,
    // contains the current active secret key for signing
    pub current: JwkKey,
}

#[derive(Debug)]
pub struct JwksKeys {
    inner: ArcSwap<JwksKeysInner>,
}

impl JwksKeys {
    pub async fn new(pool: &PgPool, settings: &Settings) -> anyhow::Result<Self> {
        Self::set_secret_key(&settings.application.master_key);
        worker::JwkCycleWorker::dispatch_at(
            pool,
            chrono::Utc::now() + chrono::Duration::seconds(settings.oauth.jwk_cycle_after_seconds),
            true,
            (),
        )
        .await?;

        let inner = Self::fetch(pool, settings).await?;
        Ok(Self {
            inner: ArcSwap::from_pointee(inner),
        })
    }

    fn set_secret_key(master_key: &SecretKey) {
        JWKS_KEY.get_or_init(|| {
            master_key.derivate(
                "24/06/2026 01:37:36 jwks key global rotation encryption v1",
                32,
            )
        });
    }

    pub fn get_current(&self) -> JwkKey {
        self.inner.load().current.clone()
    }

    pub fn get_keys(&self) -> Vec<JwkKey> {
        self.inner.load().keys.clone()
    }

    async fn update(&self, pool: &PgPool, settings: &Settings) -> anyhow::Result<()> {
        let inner = Self::fetch(pool, settings).await?;
        self.inner.store(Arc::new(inner));
        Ok(())
    }

    async fn fetch(pool: &PgPool, settings: &Settings) -> anyhow::Result<JwksKeysInner> {
        let key = JWKS_KEY
            .get()
            .expect("somehow the JWKS_KEY wasnt set before this call");
        let Some(current_jwk) = DBJwkKey::get_active(pool).await? else {
            let key = create_new_db_jwk(chrono::Utc::now(), settings)?;
            let mut tx = pool.begin().await?;
            key.insert(&mut tx).await?;
            tx.commit().await?;
            return Box::pin(Self::fetch(pool, settings)).await; // should only happen ONCE
        };

        let current_jwk_id = current_jwk.id;
        let mut retired = DBJwkKey::get_retired(pool).await?; // moved contents down there

        let mut jwks = vec![current_jwk];
        jwks.append(&mut retired);

        let mut all_jwks = Vec::new();
        let mut current_jwk = None;

        for jwk in jwks {
            let data = EncryptedSecret::new(jwk.secret, jwk.nonce);
            let decrypted_der = decrypt_secret(data, key)?;

            let mut signer = JwsEs256Signer::from_es256_der(&decrypted_der)
                .context("Failed horribly parsing decrypted jwk DER into signer")?;

            signer.set_kid(&jwk.id.to_string());
            let key = JwkKey { id: jwk.id, signer };

            if jwk.id == current_jwk_id {
                current_jwk = Some(key.clone())
            }
            all_jwks.push(key);
        }

        Ok(JwksKeysInner {
            keys: all_jwks,
            current: current_jwk
                .expect("somehow we got to return the current jwk AND its empty..."),
        })
    }
}

pub fn new_es256_key() -> anyhow::Result<Vec<u8>> {
    let key = JwsEs256Signer::generate_es256()
        .expect("Unable to generate ES256 signer")
        .private_key_to_der()?;
    Ok(key.to_vec())
}

fn create_new_db_jwk(now: chrono::DateTime<Utc>, settings: &Settings) -> anyhow::Result<DBJwkKey> {
    let secret_key = new_es256_key()?;
    let data = encrypt_secret(
        &secret_key,
        JWKS_KEY
            .get()
            .expect("JWKS_KEY must have been set at this point"),
    )?;

    let retired_at = now + chrono::Duration::seconds(settings.oauth.jwk_active_seconds);
    let expired_at = now + chrono::Duration::seconds(settings.oauth.jwk_expired_after_seconds);
    Ok(DBJwkKey::builder()
        .secret(data.secret)
        .nonce(data.nonce)
        .retired_at(retired_at)
        .max_public_age_at(expired_at)
        .build())
}
