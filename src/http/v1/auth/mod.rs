use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod flows;
mod session;
mod sudo;
mod totp;
mod webauthn;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/flow", flows::routes())
        .nest("/session", session::routes())
        .nest("/totp", totp::routes())
        .nest("/sudo", sudo::routes())
        .nest("/webauthn", webauthn::routes())
        .route("/", get(|| async { "hi there" }))
}
