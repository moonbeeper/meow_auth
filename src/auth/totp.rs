use anyhow::Context;
use data_encoding::BASE32_NOPAD;
use rand::Rng as _;
use sqlx::PgPool;
use totp_rs::Secret;

use crate::{
    crypto::{EncryptedSecret, decrypt_secret, encrypt_secret},
    database::models::{user::UserId, user_totp::UserTotp},
    settings::Settings,
};

pub fn get_totp(
    account: String,
    secret: String,
    settings: &Settings,
) -> anyhow::Result<totp_rs::TOTP> {
    let totp = totp_rs::TOTP::new(
        totp_rs::Algorithm::SHA1,
        settings.totp.digits,
        1,
        30,
        Secret::Encoded(secret).to_bytes()?,
        Some(settings.totp.issuer.clone()),
        account,
    )?;

    Ok(totp)
}

fn generate_secret(len: usize) -> Vec<u8> {
    let mut rng = rand::rng();
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    bytes
}

pub fn recovery_code(secret: &str, n: u64) -> String {
    let hash = format!("{secret}:{n}");

    blake3::hash(hash.as_bytes())
        .to_hex()
        .chars()
        .take(10)
        .collect()
}

pub fn recovery_codes(secret: &Vec<u8>) -> Vec<String> {
    let secret = hex::encode(secret);
    (0..16)
        .map(|n| recovery_code(&secret, n).to_uppercase())
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
    secret: Vec<u8>,
    recovery_secret: Vec<u8>,
    settings: &Settings,
) -> anyhow::Result<EncryptedSecrets> {
    let secret_key = settings
        .application
        .master_key
        .derivate("06/06/2026 03:10:22 totp encryption v1", 32);

    let secrets = encrypt_secret(secret.as_slice(), &secret_key).context("totp secret")?;
    let recovery_secrets =
        encrypt_secret(recovery_secret.as_slice(), &secret_key).context("totp recovery secret")?;

    Ok(EncryptedSecrets {
        secret: secrets.secret,
        secret_nonce: secrets.nonce,
        recovery_secret: recovery_secrets.secret,
        recovery_nonce: recovery_secrets.nonce,
    })
}

#[derive(Debug)]
pub struct DecryptedSecrets {
    pub secret: String,
    pub recovery_secret: Vec<u8>,
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
        secret: BASE32_NOPAD.encode(&secret),
        recovery_secret,
    })
}

pub struct CreatedTotp {
    pub model: UserTotp,
    pub secret: String,
    pub recovery_codes: Vec<String>,
}

pub async fn create_user_totp(
    user_id: UserId,
    db: &PgPool,
    settings: &Settings,
) -> anyhow::Result<CreatedTotp> {
    let secret = generate_secret(20);
    let recovery_secret = generate_secret(32);
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
        secret: BASE32_NOPAD.encode(&secret),
        recovery_codes,
    })
}

pub fn is_recovery_code_used(
    user_totp: &UserTotp,
    recovery_secret: &Vec<u8>,
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

pub fn usable_recovery_codes(user_totp: &UserTotp, recovery_secret: &Vec<u8>) -> Vec<String> {
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
    tx.commit().await?;

    Ok(())
}
