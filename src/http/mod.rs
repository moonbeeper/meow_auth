use crate::global::GlobalState;
use axum::routing::get;
use std::sync::Arc;
use tokio::net::TcpSocket;
use tokio::sync::oneshot;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

#[derive(OpenApi)]
struct ApiDocs;

fn router(global: Arc<GlobalState>) -> OpenApiRouter {
    let openapi = ApiDocs::openapi();
    OpenApiRouter::with_openapi(openapi)
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(global)
}

pub async fn run(
    global_state: Arc<GlobalState>,
    shutdown: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    // run our app with hyper, listening globally on port 3000
    let socket = TcpSocket::new_v4()?;

    socket.set_reuseaddr(true)?;
    socket.set_nodelay(true)?;

    socket.bind("0.0.0.0:3000".parse()?)?;
    let listener = socket.listen(1024)?;

    let (router, openapi) = router(global_state).split_for_parts();
    let router = router.merge(Scalar::with_url("/scalar", openapi));

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown.await;
            tracing::info!("goodnight, sweet bits and bytes...");
        })
        .await
        .expect("The HTTP server has failed its objective.");

    Ok(())
}
