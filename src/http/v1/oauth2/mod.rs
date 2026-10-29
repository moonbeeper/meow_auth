mod authorization;
mod consent;
pub mod jwks;
mod token;
mod user_info;
pub mod well_known;

use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    global::GlobalState,
    http::{cors, oauth_cors},
};

pub fn routes(global: Arc<GlobalState>) -> OpenApiRouter<Arc<GlobalState>> {
    let non_cors = OpenApiRouter::new()
        .merge(token::routes())
        .merge(jwks::routes())
        .merge(user_info::routes())
        .layer(oauth_cors());

    OpenApiRouter::new()
        .merge(authorization::routes())
        .merge(consent::routes())
        .layer(
            // url::Url forces trailing slashes. cors does not like it.
            cors(global.settings.http.frontend.as_str().trim_end_matches('/')),
        )
        .merge(non_cors)
        .route("/", get(|| async { "beep boop, i am the provider." }))
}
