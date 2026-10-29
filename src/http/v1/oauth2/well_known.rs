use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    global::GlobalState,
    http::oauth_cors,
    oauth::{
        scopes::ALL_SCOPES,
        types::{
            CodeChallengeMethod, GrantType, IdTokenSigningAlg, OauthMetadata, OpenIdMetadata,
            ResponseModes, ResponseType, SubjectTypes, TokenAuthMethod,
        },
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(wellknown_oauth))
        .routes(routes!(wellknown_openid))
        .layer(oauth_cors())
}

/// Get the oauth server metadata
#[utoipa::path(
    get,
    path = "/.well-known/oauth-authorization-server",
    tags = ["oauth_srv"],
    responses(
        (status = 200, description = "oauth authentication server metadata", body = OauthMetadata),
    )
)]
pub async fn wellknown_oauth(State(global): State<Arc<GlobalState>>) -> Json<OauthMetadata> {
    let all_scopes = ALL_SCOPES
        .iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>();

    let authorization_endpoint = make_endpoint("/v1/oauth2/authorize", &global);
    let token_endpoint = make_endpoint("/v1/oauth2/token", &global);
    let jwks_endpoint = make_endpoint("/v1/oauth2/discovery/keys", &global);
    let revocation_endpoint = make_endpoint("/v1/oauth2/revoke", &global);

    Json(OauthMetadata {
        issuer: global.settings.http.origin.to_string(),
        authorization_endpoint,
        token_endpoint,
        revocation_endpoint,
        jwks_uri: jwks_endpoint,
        scopes_supported: all_scopes,
        response_types_supported: vec![ResponseType::Code],
        response_modes_supported: vec![ResponseModes::Query],
        grant_types_supported: vec![GrantType::AuthorizationCode],
        code_challenge_methods_supported: vec![CodeChallengeMethod::S256],
    })
}

/// Get the openid server metadata
#[utoipa::path(
    get,
    path = "/.well-known/openid-configuration",
    tags = ["oauth_srv"],
    responses(
        (status = 200, description = "openid authentication server metadata", body = OpenIdMetadata),
    )
)]
pub async fn wellknown_openid(State(global): State<Arc<GlobalState>>) -> Json<OpenIdMetadata> {
    let all_scopes = ALL_SCOPES
        .iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>();

    let authorization_endpoint = make_endpoint("/v1/oauth2/authorize", &global);
    let token_endpoint = make_endpoint("/v1/oauth2/token", &global);
    let jwks_endpoint = make_endpoint("/v1/oauth2/discovery/keys", &global);
    let userinfo_endpoint = make_endpoint("/v1/oauth2/userinfo", &global);
    let revocation_endpoint = make_endpoint("/v1/oauth2/revoke", &global);

    Json(OpenIdMetadata {
        issuer: global.settings.http.origin.to_string(),
        authorization_endpoint,
        token_endpoint,
        userinfo_endpoint,
        revocation_endpoint,
        jwks_uri: jwks_endpoint,
        scopes_supported: all_scopes,
        response_types_supported: vec![ResponseType::Code],
        response_modes_supported: vec![ResponseModes::Query],
        grant_types_supported: vec![GrantType::AuthorizationCode],
        code_challenge_methods_supported: vec![CodeChallengeMethod::S256],
        subject_types_supported: vec![SubjectTypes::Public],
        id_token_signing_alg_values_supported: vec![IdTokenSigningAlg::ES256],
        token_endpoint_auth_methods_supported: vec![TokenAuthMethod::ClientSecretPost],
    })
}

fn make_endpoint(path: &str, global: &Arc<GlobalState>) -> String {
    global.settings.http.origin.join(path).unwrap().to_string()
}
