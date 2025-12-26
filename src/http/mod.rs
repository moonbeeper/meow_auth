use crate::global::GlobalState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use tokio::net::TcpSocket;
use tokio::sync::oneshot;

fn router(global: Arc<GlobalState>) -> Router {
    Router::new()
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

    let router = router(global_state);
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = shutdown.await;
            tracing::info!("goodnight, sweet bits and bytes...");
        })
        .await
        .expect("The HTTP server has failed its objective.");

    Ok(())
}
