use std::sync::Arc;

use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;

use crate::global::GlobalState;

mod flows;
mod session;
mod sudo;
mod totp;
mod webauthn;

// TODO: could merge both flow and session flows into one and then match via an enum what
// we want to do because we already check that we are working on a login flow or sudo flow

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/flow", flows::routes())
        .nest("/session", session::routes())
        .nest("/totp", totp::routes())
        .nest("/sudo", sudo::routes())
        .nest("/webauthn", webauthn::routes())
        .route("/", get(|| async { "hi there" }))
}
