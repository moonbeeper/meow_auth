mod authorization;
mod oidc;
mod token;
pub mod well_known;

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .merge(authorization::routes())
        .merge(token::routes())
}
