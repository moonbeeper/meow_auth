use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod flows;
mod sudo;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/flow", flows::routes())
        .nest("/sudo", sudo::routes())
        .route(
            "/",
            get(|| async { "hi there, welcome to... uhh, The Authentication Realm" }),
        )
}
