use axum::{
    extract::Request,
    http::{HeaderValue, header::AUTHORIZATION},
    response::Response,
};
use futures_util::future::BoxFuture;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::{
    database::models::{
        oauth_application::{OauthApplication, OauthApplicationId},
        oauth_token::OauthToken,
        user::UserId,
    },
    global::GlobalState,
    oauth::{response::OauthResponse, scopes::Scopes, secrets::hash_secret},
};

#[derive(Debug, Clone)]
pub enum OauthContext {
    Authenticated {
        user_id: UserId,
        client_id: OauthApplicationId,
        scopes: Scopes,
    },
    Unauthenticated,
}

impl OauthContext {
    pub fn is_authenticated(&self) -> bool {
        matches!(self, OauthContext::Authenticated { .. })
    }

    pub fn user_id(&self) -> UserId {
        match self {
            OauthContext::Authenticated { user_id, .. } => *user_id,
            OauthContext::Unauthenticated => UserId::nil(),
        }
    }

    pub fn client_id(&self) -> OauthApplicationId {
        match self {
            OauthContext::Authenticated { client_id, .. } => *client_id,
            OauthContext::Unauthenticated => OauthApplicationId::nil(),
        }
    }

    pub fn scopes(&self) -> Scopes {
        match self {
            OauthContext::Authenticated { scopes, .. } => *scopes,
            OauthContext::Unauthenticated => Scopes::default(),
        }
    }
}

#[derive(Clone)]
pub struct OauthManagerLayer(Arc<GlobalState>);

impl OauthManagerLayer {
    pub fn new(global: Arc<GlobalState>) -> Self {
        Self(global)
    }
}

impl<S> Layer<S> for OauthManagerLayer {
    type Service = OauthManagerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OauthManagerMiddleware {
            inner,
            global_state: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub struct OauthManagerMiddleware<S> {
    inner: S,
    global_state: Arc<GlobalState>,
}

impl<S> Service<Request> for OauthManagerMiddleware<S>
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
            // i mean, bad place BUT it should be always be set now
            OauthResponse::set_issuer(global_state.settings.http.origin.clone());
            let header = request.headers().get(AUTHORIZATION).cloned();
            if let Some(header) = &header {
                do_work(&mut request, header, &global_state).await;
            } else {
                request
                    .extensions_mut()
                    .insert(OauthContext::Unauthenticated);
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}

async fn do_work(request: &mut Request, header: &HeaderValue, global_state: &Arc<GlobalState>) {
    let token = match header.to_str() {
        Err(e) => {
            tracing::error!("failed parsing token header value: {}", e);
            request
                .extensions_mut()
                .insert(OauthContext::Unauthenticated);
            return;
        }
        Ok(v) => v
            .split_whitespace()
            .last()
            .unwrap_or("wtf, yeah Bearer (spaces). okay okay"),
    };

    let span = tracing::info_span!("oauth_manager");
    let _guard = span.enter();

    let hashed_token = hash_secret(token, &global_state.settings);

    let Ok(Some(token)) = OauthToken::find_by_token(hashed_token, &global_state.database).await
    else {
        tracing::error!("failed fetching from db or oauth token does not exist");
        request
            .extensions_mut()
            .insert(OauthContext::Unauthenticated);
        return;
    };

    // gotta fetch client to be able to remove scopes that arent anymore allowed by the client
    let Ok(Some(client)) =
        OauthApplication::find_by_id(token.client_id, &global_state.database).await
    else {
        tracing::error!("failed fetching from db or oauth client does not exist");
        request
            .extensions_mut()
            .insert(OauthContext::Unauthenticated);
        return;
    };

    let scopes = Scopes::from_bits(token.scopes).sanitize(Scopes::from_bits(client.scopes));

    request
        .extensions_mut()
        .insert(OauthContext::Authenticated {
            user_id: token.user_id,
            client_id: client.id,
            scopes,
        });
}
