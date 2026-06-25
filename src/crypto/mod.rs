pub mod jwks;

use std::{fmt::Display, ops::Deref};

use chacha20poly1305::{
    AeadCore as _, ChaCha20Poly1305, Key, KeyInit as _, Nonce,
    aead::{Aead as _, OsRng},
};
use data_encoding::BASE32_NOPAD;
use rand::Rng as _;

#[derive(Debug, Clone)]
pub struct SecretKey(pub Vec<u8>); // sadly the length can vary (apparently, see hecking totp)

impl Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0.as_slice()))
    }
}

pub fn get_secret_key(len: usize) -> SecretKey {
    let mut rng = rand::rng();
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    SecretKey(bytes)
}

impl SecretKey {
    /// Derives a new secret key from the master key.
    ///
    /// The purpose must be globally unique.
    /// A good default format for such strings is "[commit timestamp] [purpose]",
    /// e.g., "2019-12-25 16:18:03 session tokens v1".
    pub fn derivate(&self, purpose: &str, len: usize) -> SecretKey {
        // can't be used because blake3 spits out 32 bytes and we mr cookies says "NO, i want 64 NOOOW"
        // let result = blake3::derive_key(purpose, self.0.as_slice());
        let mut hasher = blake3::Hasher::new_derive_key(purpose);
        hasher.update(&self.0);
        let mut result = vec![0u8; len];
        hasher.finalize_xof().fill(&mut result);
        SecretKey(result)
    }

    pub fn as_base32(&self) -> String {
        BASE32_NOPAD.encode(self)
    }
}

impl Deref for SecretKey {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<T> AsRef<T> for SecretKey
where
    T: ?Sized,
    <SecretKey as Deref>::Target: AsRef<T>,
{
    fn as_ref(&self) -> &T {
        self.deref().as_ref()
    }
}

// tried using the newtype stuff but failed horribly. used the simpler method of "this is str"
// https://stackoverflow.com/a/65279887
impl serde::Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<SecretKey, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        Ok(SecretKey(bytes))
    }
}

pub struct EncryptedSecret {
    pub secret: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl EncryptedSecret {
    pub fn new(secret: Vec<u8>, nonce: Vec<u8>) -> Self {
        Self { secret, nonce }
    }
}

/// Encrypts a secret using ChaCha20Poly1305 with the provided secret key.
pub fn encrypt_secret(message: &[u8], secret_key: &SecretKey) -> anyhow::Result<EncryptedSecret> {
    let key = Key::from_slice(secret_key);
    let cipher = ChaCha20Poly1305::new(key);

    let secret_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message

    let secret = cipher
        .encrypt(&secret_nonce, message.as_ref())
        .map_err(|_| anyhow::anyhow!("failed encrypting the secret"))?;

    Ok(EncryptedSecret {
        secret,
        nonce: secret_nonce.to_vec(),
    })
}

/// Decrypts an encrypted secret using ChaCha20Poly1305 with the provided secret key.
pub fn decrypt_secret(
    encrypted: EncryptedSecret,
    secret_key: &SecretKey,
) -> anyhow::Result<Vec<u8>> {
    let key = Key::from_slice(secret_key);
    let cipher = ChaCha20Poly1305::new(key);

    let secret_nonce = Nonce::from_slice(&encrypted.nonce); // 96-bits; unique per message

    let secret = cipher
        .decrypt(secret_nonce, encrypted.secret.as_ref())
        .map_err(|_| anyhow::anyhow!("failed decrypting the secret"))?;

    Ok(secret)
}
