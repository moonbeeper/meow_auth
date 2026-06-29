use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod admin;
mod auth;
mod me;
pub mod oauth2;
pub mod types;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/me", me::routes())
        .nest("/oauth2", oauth2::routes())
        .nest("/admin", admin::routes())
        .route("/", get(|| async { "hiya! welcome to the v1 api :3" }))
}
