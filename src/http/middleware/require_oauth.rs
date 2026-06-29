use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::{
    http::middleware::oauth_manager::OauthContext,
    oauth::{error::OauthErrorCodes, response::OauthResponse},
};

/// this layer checks if the oauth token is valid, returning the appropriate [OauthErrorCodes] when it is not.
#[derive(Clone)]
pub struct RequireOauthLayer;

impl RequireOauthLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RequireOauthLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RequireOauthLayer {
    type Service = RequireOauth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireOauth { inner }
    }
}

#[derive(Clone)]
pub struct RequireOauth<S> {
    inner: S,
}

impl<S> Service<Request> for RequireOauth<S>
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

        Box::pin(async move {
            let context = request.extensions().get::<OauthContext>().cloned();

            if let Some(context) = &context {
                if !context.is_authenticated() {
                    return Ok(OauthResponse::new()
                        .error(OauthErrorCodes::InvalidToken, None, None)
                        .into_response());
                }
            } else {
                tracing::error!(
                    "Somehow oauth context was not set on the request extensions. are the layers in the wrong order?"
                );
                return Ok(OauthResponse::new()
                    .error(OauthErrorCodes::InvalidToken, None, None)
                    .into_response());
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}
