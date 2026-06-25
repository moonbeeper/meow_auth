use compact_jwt::{OidcSubject, OidcToken};

use crate::{
    database::models::{oauth_application::OauthApplicationId, user::User},
    oauth::{
        response::OAUTH_ISSUER,
        scopes::{Scope, Scopes},
    },
    settings::Settings,
};

pub fn get_id_token_data(
    user: User,
    client_id: OauthApplicationId,
    nonce: Option<String>,
    scopes: Scopes,
    settings: &Settings,
) -> OidcToken {
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::seconds(settings.oauth.oidc_id_token_expire_seconds);

    let mut base = OidcToken {
        iss: OAUTH_ISSUER.get().unwrap().clone(),
        sub: OidcSubject::S(user.id.to_string()),
        aud: client_id.to_string(),
        exp: expires.timestamp(),
        nbf: Some(now.timestamp()),
        iat: now.timestamp(),
        nonce,
        s_claims: Default::default(),
        acr: None,
        amr: None,
        at_hash: None,
        auth_time: Some(now.timestamp()),
        azp: None,
        claims: Default::default(),
        jti: None,
    };

    let mut standard_claims = base.s_claims.clone();

    if scopes.has(Scope::Profile) {
        standard_claims.name = Some(user.login.clone());
    }

    if scopes.has(Scope::Email) {
        standard_claims.email = Some(user.email.clone());
        standard_claims.email_verified = Some(user.email_verified);
    }

    base.s_claims = standard_claims;

    base
}
