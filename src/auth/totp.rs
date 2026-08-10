use anyhow::Context;
use sqlx::PgPool;
use totp_rs::Secret;

use crate::{
    audit::{self, AuditAction},
    crypto::{EncryptedSecret, SecretKey, decrypt_secret, encrypt_secret, get_secret_key},
    database::models::{user::UserId, user_totp::UserTotp},
    settings::Settings,
};

/// Generates a TOTP instance for a given account and secret key.
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

/// Generates a single recovery code for a given secret key and index.
pub fn get_recovery_code(secret: &str, n: u64) -> String {
    let hash = format!("{secret}:{n}");

    blake3::hash(hash.as_bytes())
        .to_hex()
        .chars()
        .take(10)
        .collect()
}

/// Generates the 16 recovery codes for a given secret key.
///
/// Normally, you would want to use the recovery codes without the separator, but for display purposes,
/// you can add the separator to make it easier to read for... people.
pub fn get_recovery_codes(secret: &SecretKey, without_separator: bool) -> Vec<String> {
    (0..16)
        .map(|n| get_recovery_code(&secret.to_string(), n).to_uppercase())
        .map(|str| {
            if without_separator {
                return str;
            }

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

/// Full stacked, create a new TOTP entry in the db for the user and returns everything needed for the user to
/// set it up, including the secret and recovery codes.
pub async fn create_user_totp(
    user_id: UserId,
    db: &PgPool,
    settings: &Settings,
) -> anyhow::Result<CreatedTotp> {
    let secret = get_secret_key(32); // n*8+4 = char len. its 52 btw. QUITE looong idk if its bad.
    let recovery_secret = get_secret_key(32);
    let recovery_codes = get_recovery_codes(&recovery_secret, false);

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TotpCodeState {
    Invalid,
    Unused(usize),
    Used(usize),
}

/// Gets the state of a given recovery code, whether its valid and unused... or plainly invalid.
///
/// If the code is valid, it will return the index inside of the enum tuple!
pub fn get_recovery_code_state(
    userdb_totp: &UserTotp,
    recovery_secret: &SecretKey,
    code: String,
) -> TotpCodeState {
    println!("the code is: {code}");
    let code = code.to_uppercase().replace("-", "");
    println!("the code now is: {code}");

    match get_recovery_codes(recovery_secret, true)
        .into_iter()
        .position(|x| x == code)
    {
        Some(idx) => {
            if userdb_totp.is_recovery_code_used(idx) {
                println!("used code");
                TotpCodeState::Used(idx)
            } else {
                println!("unsued code");

                TotpCodeState::Unused(idx)
            }
        }
        None => {
            println!("bad code");
            TotpCodeState::Invalid
        }
    }
    // .map_or_else(
    //     || (0, false),
    //     |idx| (idx, user_totp.is_recovery_code_used(idx)),
    // )
}

/// Returns the recovery codes that are still usable for a given user
pub fn get_unused_recovery_codes(user_totp: &UserTotp, recovery_secret: &SecretKey) -> Vec<String> {
    get_recovery_codes(recovery_secret, false)
        .iter()
        .enumerate()
        .filter(|(idx, _)| !user_totp.is_recovery_code_used(*idx))
        .map(|(_, v)| v.clone())
        .collect::<Vec<_>>()
}

/// Plainly, marks a recovery used by its index.
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
