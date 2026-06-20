mod authorization;
mod oidc;
pub mod well_known;

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Query, State},
    response::Response,
};
use data_encoding::BASE64URL_NOPAD;
use nom::AsBytes;
use sha2::{Digest, Sha256};
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::{
        oauth_application::OauthApplication, oauth_pending_token::OauthPendingToken,
        oauth_token::OauthToken,
    },
    global::GlobalState,
    http::{error::ApiError, middleware::auth_manager::AuthContext},
    oauth::{
        error::{OauthError, OauthErrorCodes},
        get_hashed_secret,
        scopes::Scopes,
        types::{GrantType, TokenRequest, TokenResponse},
        valid_redirect_uri, verify_client_secret,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().merge(authorization::routes())
    // .routes(routes!(token))
}

// #[axum::debug_handler]
// #[utoipa::path(
//     post,
//     path = "/token",
//     responses(
//         (status = 200, description = "current session info"),
//         (status = 500, description = "internal server error", body = ApiError)
//     )
// )]
// pub async fn token(
//     State(global): State<Arc<GlobalState>>,
//     Extension(auth): Extension<AuthContext>,
//     Json(request): Json<TokenRequest>,
// ) -> Result<Json<TokenResponse>, OauthError> {
//     if request.grant_type != GrantType::AuthorizationCode {
//         return Err(OauthError::new(
//             OauthErrorCodes::UnsupportedGrantType,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));
//         // todo!("err with UnsupportedGrantType")
//     }

//     let Ok(Some(client)) = OauthApplication::find_by_id(request.client_id, &global.database).await
//     else {
//         return Err(OauthError::new(
//             OauthErrorCodes::InvalidClient,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));

//         // todo!("err with InvalidClient")
//     };

//     let mut tx = global.database.begin().await.unwrap();
//     let Ok(Some(pending_token)) = OauthPendingToken::take_by_id(request.code, &mut tx).await else {
//         return Err(OauthError::new(
//             OauthErrorCodes::InvalidClient,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));

//         // todo!("err with InvalidClient")
//     };

//     if pending_token.client_id != client.id {
//         return Err(OauthError::new(
//             OauthErrorCodes::InvalidClient,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));

//         // todo!("err with InvalidClient")
//     }

//     let redirect_uri = Url::parse(&client.redirect_uri).unwrap();

//     if let Some(this_uri) = request.redirect_uri {
//         let request_parsed = Url::parse(&this_uri);

//         if request_parsed.is_err() {
//             return Err(OauthError::new(
//                 OauthErrorCodes::InvalidRequest,
//                 &global.settings.http.origin.clone(),
//                 &None,
//             ));

//             // todo!("err with InvalidRequest")
//         }
//         let request_parsed = request_parsed.unwrap();

//         if !valid_redirect_uri(&request_parsed, &redirect_uri) {
//             return Err(OauthError::new(
//                 OauthErrorCodes::InvalidRequest,
//                 &global.settings.http.origin.clone(),
//                 &None,
//             ));

//             // todo!("err with InvalidRequest")
//         }
//     }

//     let hashed_challenge =
//         BASE64URL_NOPAD.encode(Sha256::digest(request.code_verifier.as_bytes()).as_bytes());

//     if hashed_challenge != pending_token.code_challenge {
//         return Err(OauthError::new(
//             OauthErrorCodes::InvalidGrant,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));

//         // todo!("err with InvalidGrant")
//     }

//     if !verify_client_secret(&request.client_secret, &client.secret) {
//         return Err(OauthError::new(
//             OauthErrorCodes::InvalidClient,
//             &global.settings.http.origin.clone(),
//             &None,
//         ));

//         // todo!("err with InvalidClient")
//     }

//     let token = get_hashed_secret();
//     // re-re-sanitize scopes just innnn case :)
//     let scopes = Scopes::from_bits(pending_token.scopes).sanitize(Scopes::from_bits(client.scopes));

//     let oauth_token = OauthToken::builder()
//         .client_id(client.id)
//         .token(token.hash)
//         .user_id(pending_token.user_id)
//         .scopes(scopes.bits())
//         .build();

//     OauthToken::delete_all_by_user_and_client_id(pending_token.user_id, client.id, &mut tx)
//         .await
//         .unwrap();
//     oauth_token.insert(&mut tx).await.unwrap();
//     tx.commit().await.unwrap();

//     Ok(Json(TokenResponse {
//         access_token: token.code,
//         token_type: "Bearer".to_string(),
//         expires_in: chrono::Duration::MAX.num_seconds() as u64, // no expiry :)
//         scope: scopes.to_string(),
//         id_token: None, // id tokens for oidc... holy crap dude I actually don't have enough time to implement this
//     }))
// }

// //oidc stuff please
