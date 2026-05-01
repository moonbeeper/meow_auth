use std::sync::Arc;

use axum::{response::Html, routing::get};
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod auth;
mod me;
pub mod oauth2;
mod types;

// TODO: should have a middleware that check if the user is authenticated. (maybe?)
pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/me", me::routes())
        .nest("/oauth2", oauth2::routes())
        .route("/", get(|| async { "Hello, World!" }))
        .route("/pp", get(page))
}

async fn page() -> Html<&'static str> {
    Html("chrome devtools please")
}
