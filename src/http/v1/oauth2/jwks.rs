use std::sync::Arc;

use axum::{Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    global::GlobalState,
    oauth::types::{JwkKey, JwkKeySet},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(oauth_discovery_jwks))
}

#[utoipa::path(
    get,
    path = "/discovery/keys",
    responses(
        (status = 200, description = "oauth authentication server jwks", body = JwkKeySet),
    )
)]
pub async fn oauth_discovery_jwks(State(global): State<Arc<GlobalState>>) -> Json<JwkKeySet> {
    let all_jwks: Vec<_> = global
        .jwks
        .get_keys()
        .iter()
        .map(|v| JwkKey::from_signer(&v.signer, v.id.to_string()))
        .collect();

    Json(JwkKeySet { keys: all_jwks })
}
