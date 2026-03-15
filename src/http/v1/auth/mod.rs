use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::global::GlobalState;

mod login;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(login::login))
        .routes(routes!(login::exchange))
        .routes(routes!(login::register))
}
