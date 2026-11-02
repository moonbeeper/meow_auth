use axum::{extract::Request, http::HeaderName};
use tower_http::{
    request_id::{MakeRequestId, RequestId},
    trace::MakeSpan,
};

use crate::database::id::UlidId;

pub const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Debug, Clone, Copy, Default)]
pub struct RequestIdUlid;

impl MakeRequestId for RequestIdUlid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        Some(RequestId::new(
            UlidId::new()
                .to_string()
                .parse()
                .expect("somehow the ulid isnt a valid header value?"),
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RequestIdSpan;

impl<B> MakeSpan<B> for RequestIdSpan {
    fn make_span(&mut self, request: &Request<B>) -> tracing::Span {
        let request_id = request
            .extensions()
            .get::<RequestId>()
            .and_then(|v| v.header_value().to_str().ok())
            .unwrap_or("?");

        tracing::info_span!(
            "request",
            request_id = %request_id,
            method = %request.method(),
            uri = %request.uri(),
        )
    }
}
