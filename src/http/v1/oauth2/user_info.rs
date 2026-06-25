use std::sync::Arc;

use axum::{Json, extract::State};
use compact_jwt::Jwk;
use data_encoding::BASE64URL_NOPAD;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    global::GlobalState,
    oauth::types::{JwkKey, JwkKeySet},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(openid_userinfo))
}

#[utoipa::path(
    get,
    path = "/userinfo",
    responses(
        (status = 200, description = "oauth authentication server jwks", body = JwkKeySet),
    )
)]
pub async fn openid_userinfo(State(global): State<Arc<GlobalState>>) -> Json<JwkKeySet> {
    let all_jwks: Vec<_> = global
        .jwks
        .get_keys()
        .iter()
        .map(|v| JwkKey::from_signer(&v.signer, v.id.to_string()))
        .collect();

    Json(JwkKeySet { keys: all_jwks })
}
