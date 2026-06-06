use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use axum::{
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_valid::HasValidate;
use validator::{Validate, ValidateArgs, ValidationErrors, ValidationErrorsKind};

use crate::http::error::ApiErrorCodes;

/// !!! A wrapper around [axum_valid::lib] to provide our own error messages
#[derive(Debug)]
pub enum ValidationRejection<E> {
    /// `Valid` variant captures errors related to the validation logic.
    Valid(ValidationErrors),
    /// `Inner` variant represents potential errors that might occur within the inner extractor.
    Inner(E),
}

impl<E: Display> Display for ValidationRejection<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationRejection::Valid(errors) => write!(f, "{errors}"),
            ValidationRejection::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for ValidationRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidationRejection::Valid(ve) => Some(ve),
            ValidationRejection::Inner(e) => Some(e),
        }
    }
}

impl<E: IntoResponse> IntoResponse for ValidationRejection<E> {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::Valid(v) => {
                ApiErrorCodes::ValidationError(prettify_errors(v)).into_response()
            }
            ValidationRejection::Inner(e) => e.into_response(),
        }
    }
}

#[allow(clippy::collapsible_match, clippy::single_match)]
fn prettify_errors(errors: ValidationErrors) -> String {
    let hashmap = errors.into_errors();

    if let Some((field, kind)) = hashmap.into_iter().next() {
        match kind {
            ValidationErrorsKind::Field(e) => {
                if let Some(e) = e.first() {
                    return format!("The field '{field}' failed {} validation", e.code);
                }
            }
            // ValidationErrorsKind::Struct(_) => "struct error".to_string(),
            // ValidationErrorsKind::List(_) => "enum error".to_string(),
            _ => {}
        }
    }
    String::new()
}

/// !!! A wrapper around [axum_valid::validator] to provide our own error messages (its the same)
/// # `Valid` data extractor
///
/// This extractor can be used in combination with axum's extractors like
/// Json, Form, Query, Path, etc to validate their inner data automatically.
/// It can also work with custom extractors that implement the `HasValidate` trait.
///
/// See the docs for each integration module to find examples of using
/// `Valid` with that extractor.
///
/// For examples with custom extractors, check out the `tests/custom.rs` file.
///
#[derive(Debug, Clone, Copy, Default)]
pub struct Valid<E>(pub E);

impl<E> Deref for Valid<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Valid<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Valid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Valid<E> {
    /// Consume the `Valid` extractor and returns the inner type.
    pub fn into_inner(self) -> E {
        self.0
    }
}

/// `ValidRejection` is returned when the `Valid` or `ValidEx` extractor fails.
///
impl<E> From<ValidationErrors> for ValidationRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

/// Trait for types that can supply a reference that can be validated using arguments.
///
/// Extractor types `T` that implement this trait can be used with `ValidEx`.
///
pub trait HasValidateArgs<'v> {
    /// Inner type that can be validated using arguments
    type ValidateArgs: ValidateArgs<'v>;
    /// Get the inner value
    fn get_validate_args(&self) -> &Self::ValidateArgs;
}

impl<State, Extractor> FromRequest<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequest<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidationRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidationRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

impl<State, Extractor> FromRequestParts<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequestParts<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidationRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidationRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}
