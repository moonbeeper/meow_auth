use std::error::Error;

use axum::{
    extract::{FromRequest, OptionalFromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};

use crate::http::error::ApiErrorCodes;

/// !!! A wrapper around [axum::Json] to provide our own error messages
///
/// JSON Extractor / Response.
///
/// When used as an extractor, it can deserialize request bodies into some type that
/// implements [`serde::de::DeserializeOwned`]. The request will be rejected (and a [`JsonRejection`] will
/// be returned) if:
///
/// - The request doesn't have a `Content-Type: application/json` (or similar) header.
/// - The body doesn't contain syntactically valid JSON.
/// - The body contains syntactically valid JSON, but it couldn't be deserialized into the target type.
/// - Buffering the request body fails.
///
/// ⚠️ Since parsing JSON requires consuming the request body, the `Json` extractor must be
/// *last* if there are multiple extractors in a handler.
/// See ["the order of extractors"][order-of-extractors]
///
/// [order-of-extractors]: crate::extract#the-order-of-extractors
///
/// See [`JsonRejection`] for more details.
///
/// # Extractor example
///
/// ```rust,no_run
/// use axum::{
///     extract,
///     routing::post,
///     Router,
/// };
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUser {
///     email: String,
///     password: String,
/// }
///
/// async fn create_user(extract::Json(payload): extract::Json<CreateUser>) {
///     // payload is a `CreateUser`
/// }
///
/// let app = Router::new().route("/users", post(create_user));
/// # let _: Router = app;
/// ```
///
/// When used as a response, it can serialize any type that implements [`serde::Serialize`] to
/// `JSON`, and will automatically set `Content-Type: application/json` header.
///
/// If the [`Serialize`] implementation decides to fail
/// or if a map with non-string keys is used,
/// a 500 response will be issued
/// whose body is the error message in UTF-8.
///
/// # Response example
///
/// ```
/// use axum::{
///     extract::Path,
///     routing::get,
///     Router,
///     Json,
/// };
/// use serde::Serialize;
/// use uuid::Uuid;
///
/// #[derive(Serialize)]
/// struct User {
///     id: Uuid,
///     username: String,
/// }
///
/// async fn get_user(Path(user_id) : Path<Uuid>) -> Json<User> {
///     let user = find_user(user_id).await;
///     Json(user)
/// }
///
/// async fn find_user(user_id: Uuid) -> User {
///     // ...
///     # unimplemented!()
/// }
///
/// let app = Router::new().route("/users/{id}", get(get_user));
/// # let _: Router = app;
/// ```
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
#[must_use]
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiErrorCodes;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match <axum::Json<T> as FromRequest<S>>::from_request(req, state).await {
            Ok(v) => Ok(Json(v.0)),
            Err(e) => match e {
                JsonRejection::JsonDataError(e) => {
                    Err(ApiErrorCodes::DataError(get_inner_error(e)))
                }
                JsonRejection::JsonSyntaxError(e) => {
                    Err(ApiErrorCodes::JsonSyntaxError(get_inner_error(e)))
                }
                JsonRejection::MissingJsonContentType(_) => {
                    Err(ApiErrorCodes::MissingJsonContentType)
                }
                JsonRejection::BytesRejection(_) => Err(ApiErrorCodes::FailedToBufferContent),
                _ => Err(ApiErrorCodes::NotGood),
            },
        }
    }
}

impl<T, S> OptionalFromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiErrorCodes;

    async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
        match <axum::Json<T> as OptionalFromRequest<S>>::from_request(req, state).await {
            Ok(v) => Ok(v.map(Into::into)),
            Err(e) => match e {
                JsonRejection::JsonDataError(e) => {
                    Err(ApiErrorCodes::DataError(get_inner_error(e)))
                }
                JsonRejection::JsonSyntaxError(e) => {
                    Err(ApiErrorCodes::JsonSyntaxError(get_inner_error(e)))
                }
                JsonRejection::MissingJsonContentType(_) => {
                    Err(ApiErrorCodes::MissingJsonContentType)
                }
                JsonRejection::BytesRejection(_) => Err(ApiErrorCodes::FailedToBufferContent),
                _ => Err(ApiErrorCodes::NotGood),
            },
        }
    }
}

impl<T> From<axum::Json<T>> for Json<T> {
    fn from(value: axum::Json<T>) -> Self {
        Self(value.0)
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Json<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

// AAA:JsonSyntaxError(
//     JsonSyntaxError(
//         Error {
//             inner: Error {
//                 path: Path {
//                     segments: [
//                         Map {
//                             key: "code",
//                         },
//                     ],
//                 },
//                 original: Error("control character (\\u0000-\\u001F) found while parsing a string", line: 3, column: 0),
//             },
//         },
//     ),
// )
//
// JsonDataError(
//     Error {
//         inner: Error {
//             path: Path {
//                 segments: [
//                     Map {
//                         key: "flow_id",
//                     },
//                 ],
//             },
//             original: Error("invalid length", line: 4, column: 1),
//         },
//     },
// )

fn get_inner_error(e: impl Error) -> String {
    Error::source(&e)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
