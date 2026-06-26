use std::sync::Arc;

use axum::{extract::Form, extract::State};
use compact_jwt::JwsSigner;
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::{
        oauth_application::OauthApplication, oauth_pending_token::OauthPendingToken,
        oauth_token::OauthToken, user::User,
    },
    global::GlobalState,
    http::{error::ApiError, extractor::Json},
    oauth::{
        error::OauthErrorCodes,
        openid::get_id_token_data,
        response::OauthResponse,
        scopes::Scopes,
        secrets::{check_pkce, get_secret_pair, verify_secret},
        types::{GrantType, TokenRequest, TokenResponse, TokenType},
        valid_redirect_uri,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(token))
}

/// Exchange an authorization code for an access token
#[utoipa::path(
    post,
    path = "/token",
    tags = ["oauth_srv"],
    responses(
        (status = 200, description = "oauth token data", body = TokenResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn token(
    State(global): State<Arc<GlobalState>>,
    Form(request): Form<TokenRequest>,
) -> Result<Json<TokenResponse>, OauthResponse> {
    OauthResponse::set_issuer(global.settings.http.origin.clone());
    if request.grant_type != GrantType::AuthorizationCode {
        return Err(OauthResponse::new().error(OauthErrorCodes::UnsupportedGrantType, None, None));
    }

    // probably will have to make a custom extractor for only oauth so it can return 'InvalidRequest'
    // when there's something missing and not plain text like it does right now

    // changed my mind, will support more than 1 redirect url. will change later in models
    // if request.redirect_uri.is_none() {
    //     return Err(OauthResponse::new().error(
    //         OauthErrorCodes::InvalidRequest,
    //         Some("redirect_uri is a required param"),
    //         None,
    //     ));
    // }

    let Ok(Some(client)) = OauthApplication::find_by_id(request.client_id, &global.database).await
    else {
        return Err(OauthResponse::new().error(OauthErrorCodes::InvalidClient, None, None));
    };

    // I still hate this.
    let mut tx = global
        .database
        .begin()
        .await
        .map_err(|_| OauthResponse::new().error(OauthErrorCodes::ServerError, None, None))?;

    let Ok(Some(pending_token)) = OauthPendingToken::take_by_id(request.code, &mut tx).await else {
        return Err(OauthResponse::new().error(OauthErrorCodes::InvalidClient, None, None));
    };

    let Ok(Some(user)) = User::find_by_id(pending_token.user_id, &global.database).await else {
        return Err(OauthResponse::new().error(OauthErrorCodes::InvalidClient, None, None));
    };

    // code must be for the client_id provided
    if pending_token.client_id != client.id {
        return Err(OauthResponse::new().error(OauthErrorCodes::InvalidClient, None, None));
    }

    if !check_pkce(&request.code_verifier, &pending_token.code_challenge) {
        return Err(OauthResponse::new().error(
            OauthErrorCodes::InvalidClient,
            Some("code_verifier is invalid"),
            None,
        ));
    }

    // swap redirect_url for provided one IF its valid and matches the client one.
    let client_redirect_url = Url::parse(&client.redirect_uri).unwrap();
    let requested_url = Url::parse(&request.redirect_uri).map_err(|_| {
        OauthResponse::new().error(
            OauthErrorCodes::InvalidRequest,
            Some("redirect_uri is not a valid url"),
            None,
        )
    })?;

    // rfc says no 'InvalidRedirect' ;(
    if !valid_redirect_uri(&requested_url, &client_redirect_url) {
        return Err(OauthResponse::new().error(
            OauthErrorCodes::InvalidRequest,
            Some("redirect_uri is not a valid url"),
            None,
        ));
    }

    // even this can't use 'AccessDenied'.. and it would look reallly great here..
    if !verify_secret(&request.client_secret, &client.secret, &global.settings) {
        return Err(OauthResponse::new().error(
            OauthErrorCodes::InvalidClient,
            Some("client_secret is invalid"),
            None,
        ));
    }

    let secret_pain = get_secret_pair(&global.settings);

    // re-re-sanitize scopes just innnn case :) (great job past me!!!)
    let scopes = Scopes::from_bits(pending_token.scopes).sanitize(Scopes::from_bits(client.scopes));
    let oauth_token = OauthToken::builder()
        .client_id(client.id)
        .token(secret_pain.hash)
        .user_id(pending_token.user_id)
        .scopes(scopes.bits())
        .build();

    OauthToken::delete_all_by_user_and_client_id(pending_token.user_id, client.id, &mut tx)
        .await
        .map_err(|_| OauthResponse::new().error(OauthErrorCodes::ServerError, None, None))?;

    oauth_token
        .insert(&mut tx)
        .await
        .map_err(|_| OauthResponse::new().error(OauthErrorCodes::ServerError, None, None))?;

    // pain
    tx.commit()
        .await
        .map_err(|_| OauthResponse::new().error(OauthErrorCodes::ServerError, None, None))?;

    let mut id_token = None;
    if pending_token.is_openid {
        let current_signer = global.jwks.get_current();
        let token_data = get_id_token_data(
            user,
            pending_token.client_id,
            pending_token.nonce,
            scopes,
            &global.settings,
        );
        let token = current_signer
            .signer
            .sign(&token_data)
            .map_err(|_| OauthResponse::new().error(OauthErrorCodes::ServerError, None, None))?;
        id_token = Some(token.to_string());
    }

    Ok(Json(TokenResponse {
        access_token: secret_pain.secret,
        token_type: TokenType::Bearer,
        expires_in: chrono::Duration::MAX.num_seconds(), // no expiry :)
        scope: scopes.to_string(),
        id_token, // id tokens for oidc...
    }))
}
