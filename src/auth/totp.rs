use anyhow::Context;
use sqlx::PgPool;
use totp_rs::Secret;

use crate::{
    audit::{self, AuditAction},
    crypto::{EncryptedSecret, SecretKey, decrypt_secret, encrypt_secret, get_secret_key},
    database::models::{user::UserId, user_totp::UserTotp},
    settings::Settings,
};

pub fn get_totp(
    account: String,
    secret: SecretKey,
    settings: &Settings,
) -> anyhow::Result<totp_rs::TOTP> {
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        settings.totp.digits,
        1,
        30,
        Secret::Raw(secret.to_vec()).to_bytes()?,
        Some(settings.totp.issuer.clone()),
        account,
    )?;

    Ok(totp)
}

pub fn recovery_code(secret: &str, n: u64) -> String {
    let hash = format!("{secret}:{n}");

    blake3::hash(hash.as_bytes())
        .to_hex()
        .chars()
        .take(10)
        .collect()
}

pub fn recovery_codes(secret: &SecretKey) -> Vec<String> {
    (0..16)
        .map(|n| recovery_code(&secret.to_string(), n).to_uppercase())
        .map(|str| {
            format!(
                "{}-{}",
                str.chars().take(5).collect::<String>(),
                str.chars().skip(5).collect::<String>()
            )
        })
        .collect()
}

pub struct EncryptedSecrets {
    secret: Vec<u8>,
    secret_nonce: Vec<u8>,
    recovery_secret: Vec<u8>,
    recovery_nonce: Vec<u8>,
}

impl From<UserTotp> for EncryptedSecrets {
    fn from(value: UserTotp) -> Self {
        Self {
            secret: value.secret,
            secret_nonce: value.secret_nonce,
            recovery_secret: value.recovery_secret,
            recovery_nonce: value.recovery_secret_nonce,
        }
    }
}

// TODO: maybe could have both of these secret keys cached already with a zeroize?
fn encrypt_secrets(
    secret: SecretKey,
    recovery_secret: SecretKey,
    settings: &Settings,
) -> anyhow::Result<EncryptedSecrets> {
    let secret_key = settings
        .application
        .master_key
        .derivate("06/06/2026 03:10:22 totp encryption v1", 32);

    let secrets = encrypt_secret(&secret, &secret_key).context("totp secret")?;
    let recovery_secrets =
        encrypt_secret(&recovery_secret, &secret_key).context("totp recovery secret")?;

    Ok(EncryptedSecrets {
        secret: secrets.secret,
        secret_nonce: secrets.nonce,
        recovery_secret: recovery_secrets.secret,
        recovery_nonce: recovery_secrets.nonce,
    })
}

#[derive(Debug)]
pub struct DecryptedSecrets {
    pub secret: SecretKey,
    pub recovery_secret: SecretKey,
}

pub fn decrypt_secrets(
    encrypted: &EncryptedSecrets,
    settings: &Settings,
) -> anyhow::Result<DecryptedSecrets> {
    let secret_key = settings
        .application
        .master_key
        .derivate("06/06/2026 03:10:22 totp encryption v1", 32);

    let secret = decrypt_secret(
        EncryptedSecret {
            secret: encrypted.secret.clone(),
            nonce: encrypted.secret_nonce.clone(),
        },
        &secret_key,
    )
    .context("totp secret")?;
    let recovery_secret = decrypt_secret(
        EncryptedSecret {
            secret: encrypted.recovery_secret.clone(),
            nonce: encrypted.recovery_nonce.clone(),
        },
        &secret_key,
    )
    .context("totp recovery secret")?;

    Ok(DecryptedSecrets {
        secret: SecretKey(secret),
        recovery_secret: SecretKey(recovery_secret),
    })
}

pub struct CreatedTotp {
    pub model: UserTotp,
    pub secret: SecretKey,
    pub recovery_codes: Vec<String>,
}

pub async fn create_user_totp(
    user_id: UserId,
    db: &PgPool,
    settings: &Settings,
) -> anyhow::Result<CreatedTotp> {
    let secret = get_secret_key(32); // n*8+4 = char len. its 52 btw. QUITE looong idk if its bad.
    let recovery_secret = get_secret_key(32);
    let recovery_codes = recovery_codes(&recovery_secret);

    let secrets = encrypt_secrets(secret.clone(), recovery_secret, settings)?;

    let totp = UserTotp::builder()
        .user_id(user_id)
        .recovery_secret(secrets.recovery_secret)
        .recovery_secret_nonce(secrets.recovery_nonce)
        .secret(secrets.secret)
        .secret_nonce(secrets.secret_nonce)
        .build();
    let mut tx = db.begin().await?;
    totp.insert(&mut tx).await?;
    tx.commit().await?;

    Ok(CreatedTotp {
        model: totp,
        secret,
        recovery_codes,
    })
}

pub fn is_recovery_code_used(
    user_totp: &UserTotp,
    recovery_secret: &SecretKey,
    code: String,
) -> (usize, bool) {
    recovery_codes(recovery_secret)
        .into_iter()
        .position(|x| x == code)
        .map_or_else(
            || (0, false),
            |idx| (idx, user_totp.is_recovery_code_used(idx)),
        )
}

pub fn usable_recovery_codes(user_totp: &UserTotp, recovery_secret: &SecretKey) -> Vec<String> {
    recovery_codes(recovery_secret)
        .iter()
        .enumerate()
        .filter(|(idx, _)| !user_totp.is_recovery_code_used(*idx))
        .map(|(_, v)| v.clone())
        .collect::<Vec<_>>()
}

pub async fn set_recovery_code_used(
    idx: usize,
    user_totp: &mut UserTotp,
    db: &PgPool,
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    user_totp.mark_recovery_code_used(idx);
    user_totp.update(&mut tx).await?;
    audit::log(
        user_totp.user_id,
        user_totp.user_id,
        AuditAction::TotpRecoveryCodesUsed,
        None,
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    Ok(())
}
