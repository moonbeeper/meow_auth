use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod auth;
mod me;
pub mod oauth2;
pub mod types;

// TODO: when returning a list of something, RETURN IT INSIDE A (common) struct called "data" or something like that.
pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/me", me::routes())
        .nest("/oauth2", oauth2::routes())
        .route("/", get(|| async { "Hello, World!" }))
}
