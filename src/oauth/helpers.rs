use sqlx::PgTransaction;
use tower_cookies::Cookies;

use crate::{
    database::models::{
        oauth_application::OauthApplication, oauth_authorization::OauthAuthorization,
        oauth_pending_authorization::OauthPendingAuthorization,
        oauth_pending_token::OauthPendingToken,
    },
    http::middleware::auth_manager::AuthContext,
    oauth::{
        cookies::create_oauth_cookie, response::OAUTH_ISSUER, scopes::Scopes,
        types::AuthorizationRequest,
    },
    settings::Settings,
};

use super::types::PromptType;

#[allow(clippy::too_many_arguments)] // SHUT
pub async fn action_past_authorized(
    request: &AuthorizationRequest,
    oauth_client: OauthApplication,
    mut authorization: OauthAuthorization,
    requested_scopes: Scopes,
    mut redirect_url: url::Url,
    is_openid: bool,
    auth_context: &AuthContext,
    cookies: &Cookies,
    settings: &Settings,
    tx: &mut PgTransaction<'_>,
) -> anyhow::Result<url::Url> {
    let sanitized_authorization_scopes =
        Scopes::from_bits(authorization.scopes).sanitize(Scopes::from_bits(oauth_client.scopes));

    if is_openid && request.prompt == Some(PromptType::Consent) {
        return action_new_authorization(
            request,
            oauth_client,
            requested_scopes,
            redirect_url,
            is_openid,
            Some(authorization),
            auth_context,
            cookies,
            settings,
            tx,
        )
        .await;
    }

    if !sanitized_authorization_scopes.contains(requested_scopes) {
        let pending_auth = OauthPendingAuthorization::builder()
            .client_id(oauth_client.id)
            .code_challenge(request.code_challenge.clone())
            .user_id(auth_context.user_id())
            .state(request.state.clone())
            .nonce(request.nonce.clone())
            .user_session(auth_context.session_id())
            .requested_scopes(requested_scopes.bits())
            .old_authorization_id(Some(authorization.id))
            .old_scopes(Some(sanitized_authorization_scopes.bits()))
            .redirect_url(redirect_url.to_string())
            .is_openid(is_openid)
            .build();

        authorization.scopes = sanitized_authorization_scopes.bits();

        authorization.update(tx).await?;
        pending_auth.delete_all(tx).await?;
        pending_auth.insert(tx).await?;

        create_oauth_cookie(pending_auth.id, cookies, settings);

        // frontend would ask to the info handler for the.. info lol. That's why we are not giving it the id lol. i mean we could.
        return Ok(settings.http.frontend.join("/oauth/consent")?);
    }

    let pending_token = OauthPendingToken::builder()
        .client_id(oauth_client.id)
        .code_challenge(request.code_challenge.clone())
        .nonce(request.nonce.clone())
        .scopes(requested_scopes.bits())
        .state(request.state.clone())
        .user_id(auth_context.user_id())
        .is_openid(is_openid)
        .build();

    {
        // scoped, so mr rust isnt mad at me <:(
        let mut query_pairs = redirect_url.query_pairs_mut();
        query_pairs.append_pair("code", &pending_token.code);
        query_pairs.append_pair("iss", OAUTH_ISSUER.get().unwrap().as_ref());
        if let Some(state) = request.state.clone() {
            query_pairs.append_pair("state", &state);
        }
    }

    pending_token.delete_all(tx).await?;
    pending_token.insert(tx).await?;
    Ok(redirect_url)
}

#[allow(clippy::too_many_arguments)] // ... i know thats like. above is 9 and this is 8. but still. shut
pub async fn action_new_authorization(
    request: &AuthorizationRequest,
    oauth_client: OauthApplication,
    requested_scopes: Scopes,
    redirect_url: url::Url,
    is_openid: bool,
    past_authorization: Option<OauthAuthorization>,
    auth_context: &AuthContext,
    cookies: &Cookies,
    settings: &Settings,
    tx: &mut PgTransaction<'_>,
) -> anyhow::Result<url::Url> {
    let pending_auth = OauthPendingAuthorization::builder()
        .client_id(oauth_client.id)
        .code_challenge(request.code_challenge.clone())
        .user_id(auth_context.user_id())
        .state(request.state.clone())
        .nonce(request.nonce.clone())
        .user_session(auth_context.session_id())
        .requested_scopes(requested_scopes.bits())
        .redirect_url(redirect_url.to_string())
        .old_authorization_id(past_authorization.map(|v| v.id))
        .is_openid(is_openid)
        .build();

    pending_auth.delete_all(tx).await?;
    pending_auth.insert(tx).await?;
    create_oauth_cookie(pending_auth.id, cookies, settings);

    Ok(settings.http.frontend.join("/oauth/consent")?)
}
