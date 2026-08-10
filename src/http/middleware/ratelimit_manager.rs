use axum::{
    extract::Request,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::{
    http::{error::ApiErrorCodes, middleware::ip_manager::IpContext},
    ratelimiter::Ratelimiter,
};

/// this layer checks if the request hasn't exceeded the rate limit. Always appends the ratelimit headers to the response!
#[derive(Clone)]
pub struct RatelimitLayer(Arc<Ratelimiter>);

impl RatelimitLayer {
    pub fn new(max_tickets: u64, refill_after: chrono::Duration) -> Self {
        let ratelimiter = Ratelimiter::new(max_tickets, refill_after.to_std().unwrap());
        Self(Arc::new(ratelimiter))
    }
}

impl<S> Layer<S> for RatelimitLayer {
    type Service = Ratelimit<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Ratelimit {
            inner,
            ratelimiter: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Ratelimit<S> {
    inner: S,
    ratelimiter: Arc<Ratelimiter>,
}

impl<S> Service<Request> for Ratelimit<S>
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

    fn call(&mut self, request: Request) -> Self::Future {
        let inner_clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner_clone);
        let ratelimiter = self.ratelimiter.clone();

        Box::pin(async move {
            let context = request.extensions().get::<IpContext>().cloned();
            let ratelimit_headers: HeaderMap;

            if let Some(context) = &context {
                let response = ratelimiter.adquire(context.ip_addr());
                let headers = response.header_map();
                if !response.allowed {
                    let mut response: Response = ApiErrorCodes::RatelimitExceeded.into_response();
                    response.headers_mut().extend(headers);
                    return Ok(response);
                }
                ratelimit_headers = headers;
            } else {
                tracing::error!(
                    "Somehow the IP context was not set on the request extensions. are the layers in the wrong order?"
                );
                return Ok(ApiErrorCodes::InternalServerError.into_response());
            };

            let mut response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.

            // fixes issue where the route ratelimiter would overwrite the headers if another ratelimiter was used inside a
            // router. A clear exmaple is the /auth route, where the ratelimiter headers were overwritten with the root ones.
            // This checks if the headers are already set, and if they are not.... IT SETS THEM. gosh dang this sapce birb
            // do be thinking real stupid stuff
            for (name, value) in ratelimit_headers.iter() {
                response.headers_mut().entry(name).or_insert(value.clone());
            }
            Ok(response)
        })
    }
}
