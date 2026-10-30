use compact_jwt::{OidcSubject, OidcToken};
use time::OffsetDateTime;

use crate::{
    database::models::{oauth_application::OauthApplicationId, user::User},
    oauth::{
        response::OAUTH_ISSUER,
        scope::{BUILTIN_SCOPE_EMAIL_IDENTIFIER, BUILTIN_SCOPE_PROFILE_IDENTIFIER, Scopes},
    },
    settings::Settings,
};

pub fn get_id_token_data(
    user: User,
    client_id: OauthApplicationId,
    nonce: Option<String>,
    scopes: &Scopes,
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

    if scopes.has(BUILTIN_SCOPE_PROFILE_IDENTIFIER) {
        standard_claims.name = Some(user.name.clone());
    }

    if scopes.has(BUILTIN_SCOPE_EMAIL_IDENTIFIER) {
        standard_claims.email = Some(user.email.clone());
        standard_claims.email_verified = Some(user.email_verified);
    }

    standard_claims.updated_at = Some(
        OffsetDateTime::from_unix_timestamp(user.updated_at.timestamp())
            .expect("failed to convert user updated_at stamp to time's one"),
    );

    base.s_claims = standard_claims;

    base
}
