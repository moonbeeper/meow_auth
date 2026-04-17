use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::global::GlobalState;

mod flows;
mod session;
mod sudo;
mod totp;
mod webauthn;

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(flows::login))
        .routes(routes!(flows::exchange))
        .routes(routes!(flows::register))
        .nest("/session", session::routes())
        .nest("/totp", totp::routes())
        .nest("/sudo", sudo::routes())
        .nest("/webauthn", webauthn::routes())
}
