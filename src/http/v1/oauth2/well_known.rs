use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    global::GlobalState,
    oauth::{
        scopes::ALL_SCOPES,
        types::{CodeChallengeMethod, OauthMetadata, ResponseType},
    },
};

#[utoipa::path(
    get,
    path = "/.well-known/oauth-authorization-server",
    responses(
        (status = 200, description = "oauth authentication server metadata", body = OauthMetadata),
    )
)]
pub async fn wellknown_oauth(State(global): State<Arc<GlobalState>>) -> Json<OauthMetadata> {
    let all_scopes = ALL_SCOPES
        .iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>();
    let authorization_endpoint = format!("{}v1/oauth2/authorize", global.settings.http.origin);
    let token_endpoint = format!("{}v1/oauth2/token", global.settings.http.origin);

    Json(OauthMetadata {
        issuer: global.settings.http.origin.to_string(),
        authorization_endpoint,
        token_endpoint,
        scopes_supported: all_scopes,
        response_types_supported: vec![ResponseType::Code],
        code_challenge_methods_supported: vec![CodeChallengeMethod::S256],
    })
}

// #[utoipa::path(
//     get,
//     path = "/",
//     responses(
//         (status = 200, description = "current session info"),
//         (status = 500, description = "internal server error", body = ApiError)
//     )
// )]
// pub async fn wellknown_openid(
//     State(global): State<Arc<GlobalState>>,
//     Extension(auth): Extension<AuthContext>,
// ) -> Result<(), ApiErrorCodes> {
//     if !auth.is_authenticated() {
//         return Err(ApiErrorCodes::Unauthenticated);
//     }

//     Ok(())
// }
