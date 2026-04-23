pub mod error;
pub mod middleware;
mod v1;

use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use axum::routing::get;
use tokio::net::TcpSocket;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    global::GlobalState, http::middleware::auth_manager::AuthManagerLayer, manager::WatcherChild,
};

#[derive(OpenApi)]
struct ApiDocs;

fn router(global: Arc<GlobalState>) -> OpenApiRouter {
    let openapi = ApiDocs::openapi();
    OpenApiRouter::with_openapi(openapi)
        .route(
            "/",
            get(|| async {
                println!("HEY");
                "Hello, World!"
            }),
        )
        .nest("/v1", v1::routes())
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(AuthManagerLayer::new(global.clone()))
        .layer(CookieManagerLayer::new())
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
