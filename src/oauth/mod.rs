use argon2::{Argon2, PasswordHasher as _, PasswordVerifier as _};
use sha2::Digest;
use url::Url;

use crate::{
    database::models::{
        oauth_application::OauthApplicationId,
        oauth_pending_authorization::OauthPendingAuthorization,
    },
    http::middleware::auth_manager::AuthContext,
};

pub mod cookies;
pub mod error;
pub mod helpers;
pub mod response;
pub mod scopes;
pub mod types;

pub fn valid_redirect_uri(uri: &Url, client_uri: &Url) -> bool {
    if uri.scheme() != client_uri.scheme() {
        return false;
    }

    if uri.fragment().is_some() {
        return false;
    }

    if is_localhost(uri)
        && is_localhost(client_uri)
        && uri.scheme() == client_uri.scheme()
        && uri.host_str() == client_uri.host_str()
        && uri.path() == client_uri.path()
    {
        return true;
    }

    if uri == client_uri {
        return true;
    }

    false
}

fn is_localhost(uri: &Url) -> bool {
    match uri.host() {
        Some(url::Host::Domain("localhost")) => true,
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    }
}

pub fn pending_authorization_checks(
    pending_authorization: &OauthPendingAuthorization,
    auth: &AuthContext,
    client_id: OauthApplicationId,
) -> bool {
    if pending_authorization.user_id != auth.user_id() {
        return false;
    }

    if pending_authorization.user_session != auth.session_id() {
        return false;
    }

    if pending_authorization.client_id != client_id {
        return false;
    }

    true
}

#[derive(Debug)]
pub struct ClientSecretPair {
    pub code: String,
    pub hash: String,
}

fn generate_secret() -> String {
    nanoid::nanoid!(64)
}

pub fn get_client_secret() -> anyhow::Result<ClientSecretPair> {
    let code = generate_secret();
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(code.as_bytes())?.to_string();

    Ok(ClientSecretPair { code, hash })
}

pub fn verify_client_secret(secret: &str, hash: &str) -> bool {
    let argon2 = Argon2::default();

    let Ok(parsed_hash) = argon2::PasswordHash::new(hash) else {
        return false;
    };

    if argon2
        .verify_password(secret.as_bytes(), &parsed_hash)
        .is_err()
    {
        return false;
    }

    true
}

pub fn get_hashed_secret() -> ClientSecretPair {
    let code = generate_secret();
    let hash = hex::encode(sha2::Sha512::digest(code.as_bytes()));

    ClientSecretPair { code, hash }
}

pub fn verify_hashed_secret(secret: &str, hash: &str) -> bool {
    let hashed = hex::encode(sha2::Sha512::digest(secret.as_bytes()));

    hashed == hash
}
