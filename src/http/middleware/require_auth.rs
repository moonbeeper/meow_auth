use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::http::{error::ApiErrorCodes, middleware::auth_manager::AuthContext};

/// this layer checks if the user is authenticated or not, returning the appropriate [ApiErrorCodes].
///
/// the `need_auth` param controls whether the use needs to be authenticated or not.
///
/// `enforce` is enabled by default, so authenticated users cannot access routes meant for unauthenticated users.
///
/// disabling `enforce`, allows authenticated users to access routes meant for unauthenticated users, without returning an error.
/// **this only affects authenticated users**
#[derive(Clone)]
pub struct RequireAuthenticationLayer {
    need_auth: bool,
    enforce: bool,
}

impl Default for RequireAuthenticationLayer {
    fn default() -> Self {
        Self {
            need_auth: true,
            enforce: true,
        }
    }
}

impl RequireAuthenticationLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn need_auth(self, v: bool) -> Self {
        Self {
            need_auth: v,
            ..self
        }
    }

    pub fn enforce(self, v: bool) -> Self {
        Self { enforce: v, ..self }
    }
}

impl<S> Layer<S> for RequireAuthenticationLayer {
    type Service = RequireAuthentication<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireAuthentication {
            inner,
            need_auth: self.need_auth,
            enforce: self.enforce,
        }
    }
}

#[derive(Clone)]
pub struct RequireAuthentication<S> {
    inner: S,
    need_auth: bool,
    enforce: bool,
}

impl<S> Service<Request> for RequireAuthentication<S>
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
        let need_auth = self.need_auth;
        let enforce = self.enforce;

        Box::pin(async move {
            let context = request.extensions().get::<AuthContext>().cloned();

            if let Some(context) = &context {
                if need_auth && !context.is_authenticated() {
                    return Ok(ApiErrorCodes::Unauthenticated.into_response());
                }

                if !need_auth && context.is_authenticated() && enforce {
                    return Ok(ApiErrorCodes::AlreadyAuthenticated.into_response());
                }
            } else {
                tracing::error!(
                    "Somehow auth context was not set on the request extensions. are the layers in the wrong order?"
                );
                return Ok(ApiErrorCodes::Unauthenticated.into_response());
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}
