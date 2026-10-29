use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::{global::GlobalState, http::cors};

mod admin;
mod auth;
mod me;
pub mod oauth2;
pub mod types;

pub fn routes(global: Arc<GlobalState>) -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/auth", auth::routes())
        .nest("/me", me::routes())
        .nest("/admin", admin::routes())
        .route("/", get(|| async { "hiya! welcome to the v1 api :3" }))
        .layer(
            // url::Url forces trailing slashes. cors does not like it.
            cors(global.settings.http.frontend.as_str().trim_end_matches('/')),
        )
        .nest("/oauth2", oauth2::routes(global.clone()))
}
