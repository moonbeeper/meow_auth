mod users;

use std::sync::Arc;

use utoipa_axum::router::OpenApiRouter;

use crate::{
    auth::flags::UserFlag,
    global::GlobalState,
    http::middleware::{
        require_auth::RequireAuthenticationLayer, require_user_flag::RequireUserFlagLayer,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .nest("/users", users::routes())
        .layer(RequireAuthenticationLayer::new())
        .layer(RequireUserFlagLayer::new().require(UserFlag::SuperAdmin))
}
