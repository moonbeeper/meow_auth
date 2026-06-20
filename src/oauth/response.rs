use std::sync::OnceLock;

use axum::response::{IntoResponse, Redirect};

use crate::oauth::error::{OauthError, OauthErrorCodes};

pub static OAUTH_ISSUER: OnceLock<url::Url> = OnceLock::new();

#[derive(Debug, Default)]
pub struct OauthResponse {
    redirect: Option<String>,
    error: Option<ErrorResponse>,
}

#[derive(Debug)]
struct ErrorResponse {
    code: OauthErrorCodes,
    description: Option<&'static str>,
    state: Option<String>,
}

impl OauthResponse {
    pub fn new() -> Self {
        Self::default()
    }

    // must be used before any OauthResponse is created, otherwise the whole handler will shit it self
    // also cannot be changed after set
    pub fn set_issuer(url: url::Url) {
        let _ = OAUTH_ISSUER.get_or_init(|| url);
    }

    pub fn redirect(mut self, url: String) -> Self {
        self.redirect = Some(url);
        self
    }

    pub fn error(
        mut self,
        code: OauthErrorCodes,
        description: Option<&'static str>,
        state: Option<String>,
    ) -> Self {
        self.error = Some(ErrorResponse {
            code,
            description,
            state,
        });
        self
    }
}

// redirect uri should always be correct, it should be already validated when added to a client
impl IntoResponse for OauthResponse {
    fn into_response(self) -> axum::response::Response {
        let issuer = OAUTH_ISSUER.get().unwrap();
        if let Some(error_meta) = self.error {
            let description = match error_meta.description {
                Some(v) => v,
                None => error_meta.code.description(),
            };
            if let Some(redirect_url) = self.redirect {
                let mut url = url::Url::parse(&redirect_url).unwrap();
                {
                    // scoped, so mr rust isnt mad at me <:(
                    let mut query_pairs = url.query_pairs_mut();
                    query_pairs.append_pair("error", error_meta.code.as_str());
                    query_pairs.append_pair("error_description", description);
                    query_pairs.append_pair("iss", issuer.as_ref());

                    if let Some(state) = error_meta.state {
                        query_pairs.append_pair("state", &state);
                    }
                }
                return Redirect::to(url.as_ref()).into_response();
            }

            let error = OauthError::new(
                error_meta.code,
                issuer,
                error_meta.description.map(|v| v.to_string()),
                &error_meta.state,
            );
            return error.into_response();
        }

        if let Some(redirect_url) = self.redirect {
            return Redirect::to(&redirect_url).into_response();
        }

        // fallback if somehow i did not set a redirect or error. shouldnt happen tho
        (
            axum::http::StatusCode::PRECONDITION_FAILED,
            "no redirect or error set",
        )
            .into_response()
    }
}
