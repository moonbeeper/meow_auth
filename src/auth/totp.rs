use chacha20poly1305::{
    AeadCore as _, ChaCha20Poly1305, Key, KeyInit as _, Nonce,
    aead::{Aead as _, OsRng},
};
use data_encoding::BASE32_NOPAD;
use rand::Rng as _;
use sqlx::PgPool;
use totp_rs::Secret;

use crate::{
    database::models::{user::UserId, user_totp::UserTotp},
    settings::Settings,
};

pub fn make_totp(
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

fn encrypt_secrets(
    secret: Vec<u8>,
    recovery_secret: Vec<u8>,
    settings: &Settings,
) -> anyhow::Result<EncryptedSecrets> {
    let key = hex::decode(settings.totp.encryption_secret.clone())?;
    let key = Key::from_slice(&key);
    let cipher = ChaCha20Poly1305::new(key);

    let secret_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let recovery_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let secret = cipher
        .encrypt(&secret_nonce, secret.as_ref())
        .map_err(|_| anyhow::anyhow!("failed encrypting the secret"))?;
    let recovery_secret = cipher
        .encrypt(&recovery_nonce, recovery_secret.as_ref())
        .map_err(|_| anyhow::anyhow!("failed encrypting the recovery secret"))?;

    Ok(EncryptedSecrets {
        secret,
        secret_nonce: secret_nonce.to_vec(),
        recovery_secret,
        recovery_nonce: recovery_nonce.to_vec(),
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
    let key = hex::decode(settings.totp.encryption_secret.clone())?;
    let key = Key::from_slice(&key);
    let cipher = ChaCha20Poly1305::new(key);

    let secret_nonce = Nonce::from_slice(&encrypted.secret_nonce); // 96-bits; unique per message
    let recovery_nonce = Nonce::from_slice(&encrypted.recovery_nonce); // 96-bits; unique per message

    let secret = cipher
        .decrypt(secret_nonce, encrypted.secret.as_ref())
        .map_err(|_| anyhow::anyhow!("failed decrypting the secret"))?;

    let recovery_secret = cipher
        .decrypt(recovery_nonce, encrypted.recovery_secret.as_ref())
        .map_err(|_| anyhow::anyhow!("failed decrypting the recovery secret"))?;

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
