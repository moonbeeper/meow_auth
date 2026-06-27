pub mod error;
pub mod extractor;
pub mod middleware;
mod root;
mod v1;
pub mod validator;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use tokio::net::TcpSocket;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    global::GlobalState,
    http::middleware::{auth_manager::AuthManagerLayer, oauth_manager::OauthManagerLayer},
    manager::WatcherChild,
};

#[derive(OpenApi)] // my dumb heck thought that IT was inside the derive macro where you set the attribute tags smh.
#[openapi(
    tags(
        (name = "auth", description = "authentication (login, register)"),
        (name = "sudo", description = "sensitive operations re-authentication"),
        (name = "user", description = "current user operations"),
        (name = "totp", description = "two-factor authentication management"),
        (name = "passkeys", description = "passkey authentication management"),
        (name = "sessions", description = "session management"),
        (name = "application", description = "application health and status"),
        (name = "oauth", description = "oauth management operations"),
        (name = "oauth_srv", description = "oauth server operations"),
    )
)]
struct ApiDocs;

fn router(global: Arc<GlobalState>) -> OpenApiRouter {
    let openapi = ApiDocs::openapi();
    OpenApiRouter::with_openapi(openapi)
        .nest("/v1", v1::routes())
        // .routes(routes!(v1::oauth2::well_known::wellknown_oauth))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(AuthManagerLayer::new(global.clone()))
        .layer(OauthManagerLayer::new(global.clone()))
        .layer(CookieManagerLayer::new())
        // below the auth manager layer so we don't gotta check for auth (useless) on these static handlers
        .merge(root::routes())
        .with_state(global)
    // middlewares go from bottom to top for requests, and top to bottom for responses.
}

pub async fn run(global: Arc<GlobalState>, watcher: WatcherChild) -> anyhow::Result<()> {
    let settings = global.settings.http.clone();
    let socket = match settings.bind {
        SocketAddr::V4(_) => TcpSocket::new_v4()?,
        SocketAddr::V6(_) => TcpSocket::new_v6()?,
    };

    socket.set_reuseaddr(true)?;
    socket.set_nodelay(true)?;
    socket.set_reuseport(true)?;

    socket.bind(settings.bind)?;
    let listener = socket.listen(1024)?;

    tracing::info!("HTTP server listening at http://{}", settings.bind);

    let (router, openapi) = router(global).split_for_parts();
    let router = match settings.api_docs.enabled {
        true => {
            tracing::info!("OpenApi documentation at http://{}/scalar", settings.bind);
            router.merge(Scalar::with_url(settings.api_docs.path, openapi))
        }
        false => router,
    };

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            watcher.cancelled().await;
            tracing::info!("goodnight, sweet bits and flying toasters with wings");
        })
        .await
        .context("Failed starting the HTTP server")?;

    Ok(())
}
