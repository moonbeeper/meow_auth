use axum::{
    extract::{ConnectInfo, Request},
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::{global::GlobalState, http::error::ApiErrorCodes};

#[derive(Debug, Clone)]
pub struct IpContext(IpAddr);

impl IpContext {
    pub fn ip_addr(&self) -> IpAddr {
        self.0
    }
}

/// this layer simply adds the connecting ip address to the request extensions
#[derive(Clone)]
pub struct IpManagerLayer(Arc<GlobalState>);

impl IpManagerLayer {
    pub fn new(global: Arc<GlobalState>) -> Self {
        Self(global)
    }
}

impl<S> Layer<S> for IpManagerLayer {
    type Service = IpManager<S>;

    fn layer(&self, inner: S) -> Self::Service {
        IpManager {
            inner,
            global_state: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub struct IpManager<S> {
    inner: S,
    global_state: Arc<GlobalState>,
}

impl<S> Service<Request> for IpManager<S>
where
    S: Service<Request, Response = Response> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let inner_clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);
        let global_state = self.global_state.clone();

        Box::pin(async move {
            let context = request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .cloned();

            if let Some(ip_addr) = &context {
                let mut ip_addr = ip_addr.ip().to_canonical();

                #[allow(clippy::collapsible_if)]
                if global_state.settings.http.is_under_cloudflare {
                    if let Some(cf_ip) = request.headers().get("CF-Connecting-IP") {
                        let cf_ip = cf_ip.to_str().unwrap();
                        if let Ok(cf_ip) = cf_ip.parse::<IpAddr>() {
                            ip_addr = cf_ip.to_canonical();
                        }
                    }
                }

                request.extensions_mut().insert(IpContext(ip_addr));
            } else {
                tracing::error!(
                    "Somehow the connecting ip addr is absent from the request extensions."
                );
                return Ok(ApiErrorCodes::InternalServerError.into_response());
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}
