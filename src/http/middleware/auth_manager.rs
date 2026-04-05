use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use std::{
    sync::Arc,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tower_cookies::Cookies;

use crate::{
    auth::session::{delete_session_cookie, get_session_cookie, renew_session},
    database::models::{
        user::UserId,
        user_session::{PIDUserSessionId, UserSession, UserSessionId},
    },
    global::GlobalState,
};

#[derive(Debug, Clone)]
pub enum AuthContext {
    Authenticated {
        user_id: UserId,
        session_id: UserSessionId,
        is_sudo_enabled: bool,
    },
    Unauthenticated,
}

impl AuthContext {
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthContext::Authenticated { .. })
    }

    pub fn user_id(&self) -> UserId {
        match self {
            AuthContext::Authenticated { user_id, .. } => *user_id,
            AuthContext::Unauthenticated => UserId::nil(),
        }
    }

    pub fn session_id(&self) -> UserSessionId {
        match self {
            AuthContext::Authenticated { session_id, .. } => *session_id,
            AuthContext::Unauthenticated => UserSessionId::nil(),
        }
    }

    pub fn is_sudo_enabled(&self) -> bool {
        match self {
            AuthContext::Authenticated {
                is_sudo_enabled: is_sudo,
                ..
            } => *is_sudo,
            AuthContext::Unauthenticated => false,
        }
    }
}

#[derive(Clone)]
pub struct AuthManagerLayer(Arc<GlobalState>);

impl AuthManagerLayer {
    pub fn new(global: Arc<GlobalState>) -> Self {
        Self(global)
    }
}

impl<S> Layer<S> for AuthManagerLayer {
    type Service = AuthManagerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthManagerMiddleware {
            inner,
            global_state: self.0.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthManagerMiddleware<S> {
    inner: S,
    global_state: Arc<GlobalState>,
}

impl<S> Service<Request> for AuthManagerMiddleware<S>
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
            let cookies = request.extensions().get::<Cookies>().cloned();
            if let Some(cookies) = &cookies {
                do_work(&mut request, cookies, &global_state).await;
            } else {
                tracing::error!(
                    "Somehow cookies were not set on the request extensions. are the layers in the wrong order?"
                );
                request
                    .extensions_mut()
                    .insert(AuthContext::Unauthenticated);
            };

            let response: Response = inner.call(request).await?; // <-- goes into handler then makes the response.
            Ok(response)
        })
    }
}

async fn do_work(request: &mut Request, cookies: &Cookies, global_state: &Arc<GlobalState>) {
    let Some(cookie) = get_session_cookie(cookies, &global_state.settings) else {
        // tracing::info!("no cookies for me :(");
        request
            .extensions_mut()
            .insert(AuthContext::Unauthenticated);
        return;
    };

    // TODO: I could be using directly the Id from the db instead of the PID because of the use of encrypted cookies.
    let pid = match cookie.value().parse::<PIDUserSessionId>() {
        Ok(pid) => pid,
        Err(e) => {
            tracing::error!("failed parsing PID cookie value: {}", e);
            delete_session_cookie(cookies, &global_state.settings);
            request
                .extensions_mut()
                .insert(AuthContext::Unauthenticated);
            return;
        }
    };

    let span = tracing::info_span!("auth_manager", pid = %pid);
    let _guard = span.enter();

    let Ok(Some(mut session)) = UserSession::find_by_pid(pid, &global_state.database).await else {
        tracing::error!("failed fetching from db or session does not exist");
        delete_session_cookie(cookies, &global_state.settings);
        request
            .extensions_mut()
            .insert(AuthContext::Unauthenticated);
        return;
    };

    let now = chrono::Utc::now();

    // session is expired
    if session.expires_at <= now {
        tracing::info!("session is expired");
        delete_session_cookie(cookies, &global_state.settings);
        request
            .extensions_mut()
            .insert(AuthContext::Unauthenticated);
        return;
    }

    // session is active
    if now <= session.active_expires_at {
        tracing::info!("session is active, checking if can renew active expires");

        let update_threshold =
            chrono::Duration::seconds(global_state.settings.session.update_threshold_seconds);
        if now - session.updated_at >= update_threshold {
            tracing::info!("will renew active session expire");
            if let Err(e) =
                renew_session(&mut session, &global_state.database, &global_state.settings).await
            {
                tracing::error!("failed renewing session: {}", e);
                delete_session_cookie(cookies, &global_state.settings);
                request
                    .extensions_mut()
                    .insert(AuthContext::Unauthenticated);
                return;
            } else {
                tracing::info!("successfully renewed session")
            }
        }

        request.extensions_mut().insert(AuthContext::Authenticated {
            user_id: session.user_id,
            session_id: session.id,
            is_sudo_enabled: session.is_sudo_enabled(),
        });
        return;
    }

    // session is inactive but not expired
    if session.active_expires_at <= now {
        tracing::info!("session is inactive, updating active expires",);

        tracing::info!("will renew active session expire");
        if let Err(e) =
            renew_session(&mut session, &global_state.database, &global_state.settings).await
        {
            tracing::error!("failed renewing session: {}", e);
            delete_session_cookie(cookies, &global_state.settings);
            request
                .extensions_mut()
                .insert(AuthContext::Unauthenticated);
            return;
        } else {
            tracing::info!("successfully renewed session")
        }

        request.extensions_mut().insert(AuthContext::Authenticated {
            user_id: session.user_id,
            session_id: session.id,
            is_sudo_enabled: session.is_sudo_enabled(),
        });
    }
}
