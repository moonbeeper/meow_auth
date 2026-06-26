mod authorization;
mod consent;
pub mod jwks;
mod token;
mod user_info;
pub mod well_known;

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .merge(authorization::routes())
        .merge(token::routes())
        .merge(jwks::routes())
        .merge(user_info::routes())
        .merge(consent::routes())
}
