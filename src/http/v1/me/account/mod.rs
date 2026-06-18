mod email;

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().merge(email::routes())
}
