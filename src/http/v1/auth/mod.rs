use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod login;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
}
