use axum::{
    extract::Request,
    http::header::USER_AGENT,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::http::error::ApiErrorCodes;

#[derive(Debug, Clone)]
pub struct UserAgentContext(String);

impl UserAgentContext {
    pub fn agent(&self) -> String {
        self.0.clone()
    }
}

/// this layer's job is to just extract the user agent from the request and add it to the request extensions.
/// Also forces the user agent to be present, otherwise it returns a bad request error.
#[derive(Clone)]
pub struct UserAgentManagerLayer;

// i wondered about adding a UA parser to get "WINDOWS" "HELLIUM BROWSERE" stuff like that but... can't the frontend
// do the parsing for me? I think it can? I mean its just getting a fricking package and boom duck tape
#[allow(clippy::new_without_default)]
impl UserAgentManagerLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for UserAgentManagerLayer {
    type Service = UserAgentManager<S>;

    fn layer(&self, inner: S) -> Self::Service {
        UserAgentManager { inner }
    }
}

#[derive(Clone)]
pub struct UserAgentManager<S> {
    inner: S,
}

impl<S> Service<Request> for UserAgentManager<S>
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

        Box::pin(async move {
            let user_agent = request
                .headers()
                .get(USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned);

            if let Some(ua) = &user_agent {
                request
                    .extensions_mut()
                    .insert(UserAgentContext(ua.to_string()));
            } else {
                // TODO: i dont know... maybe this isnt the right thing to do. Like maybe i should just let it go
                // thorugh and that's it... hm
                return Ok(ApiErrorCodes::MissingUserAgent.into_response());
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}
