use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::global::GlobalState;

mod flows;
mod session;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(flows::login))
        .routes(routes!(flows::exchange))
        .routes(routes!(flows::register))
        .nest("/session", session::routes())
}
