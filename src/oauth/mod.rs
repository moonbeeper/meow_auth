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
pub mod openid;
pub mod response;
pub mod scopes;
pub mod secrets;
pub mod types;

// TODO: add Query, Form extractors to return Oauth errors

pub fn valid_redirect_uri(uri: &Url, client_uri: &Url) -> bool {
    if !valid_uri(uri) || !valid_uri(client_uri) {
        return false;
    }

    if uri.scheme() != client_uri.scheme() {
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

const VALID_URI_SCHEMES: [&str; 2] = ["http", "https"];

/// validates if the provided url is a valid uri for redirecting to. IT HAS no fragments or queries.
pub fn valid_uri(url: &Url) -> bool {
    if !VALID_URI_SCHEMES.contains(&url.scheme()) {
        return false;
    }

    if url.fragment().is_some() {
        return false;
    }

    if url.query().is_some() {
        return false;
    }

    if is_localhost(url) {
        return true;
    }

    true
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
