use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::{
    auth::flags::{UserFlag, UserFlags},
    http::{error::ApiErrorCodes, middleware::auth_manager::AuthContext},
};

/// this layer checks if the user has the required or forbidden flags, returning the appropriate [ApiErrorCodes].
///
/// This is handy when you want to gate features behind flags, like a "management" feature
#[derive(Clone, Default)]
pub struct RequireUserFlagLayer {
    require: UserFlags,
    forbid: UserFlags,
}

impl RequireUserFlagLayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require(mut self, flag: UserFlag) -> Self {
        self.require = self.require.add(flag);
        self
    }

    pub fn forbid(mut self, flag: UserFlag) -> Self {
        self.forbid = self.forbid.add(flag);
        self
    }
}

impl<S> Layer<S> for RequireUserFlagLayer {
    type Service = RequireUserFlag<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequireUserFlag {
            inner,
            require: self.require,
            forbid: self.forbid,
        }
    }
}

#[derive(Clone)]
pub struct RequireUserFlag<S> {
    inner: S,
    require: UserFlags,
    forbid: UserFlags,
}

impl<S> Service<Request> for RequireUserFlag<S>
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
        let required_flags = self.require;
        let forbid_flags = self.forbid;

        Box::pin(async move {
            let context = request.extensions().get::<AuthContext>().cloned();

            if let Some(context) = &context {
                if !context.is_authenticated() {
                    return Ok(ApiErrorCodes::Unauthenticated.into_response());
                }

                let user_flags = context.user_flags();

                if (!required_flags.is_empty() && !user_flags.contains(required_flags))
                    || (!forbid_flags.is_empty() && user_flags.contains(forbid_flags))
                {
                    return Ok(ApiErrorCodes::ActionBlocked.into_response());
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
