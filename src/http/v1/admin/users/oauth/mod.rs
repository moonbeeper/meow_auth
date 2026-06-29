mod application;
mod authorization;

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/application", application::routes())
        .nest("/authorization", authorization::routes())
}
