use axum::{
    extract::{ConnectInfo, Request},
    response::{IntoResponse, Response},
};
use futures_util::future::BoxFuture;
use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};
use tower::{Layer, Service};

use crate::{global::GlobalState, http::error::ApiErrorCodes};

static LOCATION_BAD_SWITCH: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct IpContext {
    ip_addr: IpAddr,
    location: EitherLocation,
}

#[derive(Debug, Clone, Default)]
pub enum EitherLocation {
    #[default]
    Unknown,
    Known(IpLocation),
}

#[derive(Debug, Clone)]
pub struct IpLocation {
    pub country_code: String,
    pub city: String,
}

impl Display for EitherLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let EitherLocation::Known(v) = self {
            write!(f, "{}, {}", v.city, v.country_code)
        } else {
            write!(f, "Unknown")
        }
    }
}

impl IpContext {
    pub fn ip_addr(&self) -> IpAddr {
        self.ip_addr
    }

    pub fn location(&self) -> String {
        self.location.to_string()
    }

    pub fn raw_location(&self) -> EitherLocation {
        self.location.clone()
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
                let mut location = EitherLocation::Unknown;

                #[allow(clippy::collapsible_if)]
                if global_state.settings.http.is_under_cloudflare {
                    // always added by cloudflare
                    let cf_ip = request
                        .headers()
                        .get("CF-Connecting-IP")
                        .and_then(|v| v.to_str().ok());

                    // added if the "Add visitor location headers" option is enabled on the cloudflare dashboard
                    let cf_ipcity = request
                        .headers()
                        .get("cf-ipcity")
                        .and_then(|v| v.to_str().ok());

                    let cf_ipcountry = request
                        .headers()
                        .get("cf-ipcountry")
                        .and_then(|v| v.to_str().ok());

                    if let Some(cf_ip) = cf_ip {
                        if let Ok(cf_ip) = cf_ip.parse::<IpAddr>() {
                            ip_addr = cf_ip.to_canonical();
                        }
                    }

                    if let (Some(city), Some(country)) = (cf_ipcity, cf_ipcountry) {
                        location = EitherLocation::Known(IpLocation {
                            country_code: country.to_string(),
                            city: city.to_string(),
                        })
                    } else if !LOCATION_BAD_SWITCH.get().is_some() {
                        LOCATION_BAD_SWITCH.get_or_init(|| ());
                        // ip has a fallback. i frankly dont care if the ip is cloudflare's ips or not. BUT these DO show some misconfiguration
                        tracing::error!(
                            "'is_under_cloudflare' is set but we can't find the 'cf-ipcity' or 'cf-ipcountry' headers! Are we really under cloudflare? Did you enable 'Add visitor location headers' (under Rules/Settings) on the dashboard?"
                        )
                    }
                }

                request
                    .extensions_mut()
                    .insert(IpContext { ip_addr, location });
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
