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
use tower_cookies::Cookies;
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::{
        oauth_application::OauthApplication, oauth_authorization::OauthAuthorization,
        oauth_pending_authorization::OauthPendingAuthorization,
        oauth_pending_token::OauthPendingToken, oauth_token::OauthToken,
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
    },
    oauth::{
        cookies::{create_oauth_cookie, get_pending_authorization_id_from_cookies},
        error::{OauthError, OauthErrorCodes, redirect_to, redirect_with_error},
        get_hashed_secret, pending_authorization_checks,
        scopes::Scopes,
        types::{
            AuthorizationFinishRequest, AuthorizationRequest, CodeChallengeMethod, GrantType,
            ResponseType, TokenRequest, TokenResponse,
        },
        valid_redirect_uri, verify_client_secret,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(authorize))
        .routes(routes!(finish_authorization))
        .routes(routes!(token))
}
#[axum::debug_handler]
#[utoipa::path(
    get,
    path = "/authorize",
    params(AuthorizationRequest),
    responses(
        (status = 303, description = "redirect to consent screen or redirect_uri with code"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn authorize(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Query(request): Query<AuthorizationRequest>,
) -> Response {
    let fallback_iss = global.settings.http.origin.clone();

    let state = request.state.clone();
    let just_work_pelase = request.clone();
    let redirect_err = |url: &Url, code: OauthErrorCodes| {
        redirect_with_error(
            url.clone(),
            OauthError::new(code, &global.settings.http.origin, &state),
        )
    };

    // must be authorization_code
    if request.response_type != ResponseType::Code {
        return redirect_err(&fallback_iss, OauthErrorCodes::UnsupportedResponseType);
        // todo!("err with UnsupportedResponseType")
    }

    if request.code_challenge_method != CodeChallengeMethod::S256 {
        return redirect_err(&fallback_iss, OauthErrorCodes::InvalidRequest);

        // todo!("err with InvalidRequest")
    }

    let Ok(Some(client)) = OauthApplication::find_by_id(request.client_id, &global.database).await
    else {
        return redirect_err(&fallback_iss, OauthErrorCodes::InvalidClient);

        // todo!("err with InvalidClient")
    };

    let mut redirect_uri = Url::parse(&client.redirect_uri).unwrap();

    if let Some(this_uri) = request.redirect_uri {
        let request_parsed = Url::parse(&this_uri);

        if request_parsed.is_err() {
            return redirect_err(&fallback_iss, OauthErrorCodes::InvalidRequest);

            // todo!("err with InvalidRequest")
        }
        let request_parsed = request_parsed.unwrap();

        if !valid_redirect_uri(&request_parsed, &redirect_uri) {
            return redirect_err(&fallback_iss, OauthErrorCodes::InvalidRedirect);

            // todo!("err with InvalidRedirect")
        }
        redirect_uri = request_parsed
    }

    let mut current_scopes = Scopes::from_bits(client.scopes); // max client scopes

    if let Some(scope_str) = request.scope {
        let requested = Scopes::from_str(&scope_str, true).unwrap();

        // if requesting too many scopes stop
        if !current_scopes.contains(requested) {
            return redirect_err(&fallback_iss, OauthErrorCodes::InvalidScope);

            // todo!("err with InvalidScope")
        }
        current_scopes = requested;
    }

    if !auth.is_authenticated() {
        return redirect_err(&fallback_iss, OauthErrorCodes::AccessDenied);
    }

    let Ok(authorization) =
        OauthAuthorization::find_by_user_and_client_id(auth.user_id(), client.id, &global.database)
            .await
    else {
        return redirect_err(&fallback_iss, OauthErrorCodes::ServerError);

        // todo!("err with ServerError")
    };

    let Ok(result) = create_pending_authorization_and_stuff(
        just_work_pelase,
        auth,
        client,
        authorization,
        current_scopes,
        &cookies,
        &global,
    )
    .await
    else {
        return redirect_err(&fallback_iss, OauthErrorCodes::ServerError);
    };

    result
}

// out of time for making it nice and clean. ugly way it is... thanks mr my procrastination for flavortown
async fn create_pending_authorization_and_stuff(
    request: AuthorizationRequest,
    auth: AuthContext,
    client: OauthApplication,
    authorization: Option<OauthAuthorization>,
    current_scopes: Scopes,
    cookies: &Cookies,
    global: &Arc<GlobalState>,
) -> anyhow::Result<Response> {
    let fallback_iss = global.settings.http.origin.clone();

    // aaa man fuckin ghell gross as heck. cloens stupid stuff and god dude i hate it so much now it was rpetty now it snot
    let base_pending = OauthPendingAuthorization::builder()
        .client_id(client.id)
        .code_challenge(request.code_challenge.clone())
        .user_id(auth.user_id())
        .state(request.state.clone())
        .nonce(request.nonce.clone())
        .user_session(auth.session_id())
        .requested_scopes(current_scopes.bits());

    if let Some(mut authorization) = authorization {
        let authorization_scopes = Scopes::from_bits(authorization.scopes);
        let authorization_scopes = authorization_scopes.sanitize(Scopes::from_bits(client.scopes)); // remove any scopes that the client no longer has

        let mut tx = global.database.begin().await?;

        // if the authorization doesnt have the requested scopes, consent
        if !authorization_scopes.contains(current_scopes) {
            let this_pending = base_pending
                .old_scopes(Some(authorization_scopes.bits()))
                .build();

            authorization.scopes = authorization_scopes.bits(); // update with sanitized scopes
            authorization.update(&mut tx).await?;

            this_pending.delete_all(&mut tx).await?;
            this_pending.insert(&mut tx).await?;
            tx.commit().await?;

            create_oauth_cookie(this_pending.id, &cookies, &global.settings);

            let joied_str = format!(
                "/todo/oauth_consent_id/{:?}/client_id/{:?}",
                this_pending.id, client.id
            );
            return Ok(redirect_to(&fallback_iss.join(&joied_str)?));
            // todo!("1. redirect to consent screen with flow attached to user session and scopes")
        }

        // // update last used because its the same stuff!
        // authorization.update(&mut tx).await.unwrap();
        // tx.commit().await.unwrap();

        // holy shit this is gorss. GOD MAN WHY I PROCRASTINATE SO MUCH LIKE DUDE WHY
        let pending_token = OauthPendingToken::builder()
            .client_id(client.id)
            .user_id(auth.user_id())
            .nonce(request.nonce.clone())
            .state(request.state.clone())
            .code_challenge(request.code_challenge.clone())
            .scopes(current_scopes.bits())
            .build();

        let mut tx = global.database.begin().await?;
        authorization.update(&mut tx).await?;
        pending_token.insert(&mut tx).await?;
        tx.commit().await?;

        let mut url = Url::parse(&client.redirect_uri)?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("code", &pending_token.code);
            if let Some(state) = request.state {
                q.append_pair("state", &state);
            }
            q.append_pair("iss", fallback_iss.as_str());
        }
        return Ok(redirect_to(&url));

        // todo!("redirect to redirect_uri with oauth stuff");
    }

    let this_pending = base_pending.build();

    let mut tx = global.database.begin().await.unwrap();
    this_pending.delete_all(&mut tx).await.unwrap();
    this_pending.insert(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    create_oauth_cookie(this_pending.id, &cookies, &global.settings);
    let joied_str = format!(
        "/todo/oauth_consent_id/{:?}/client_id/{:?}",
        this_pending.id, client.id
    );
    return Ok(redirect_to(&fallback_iss.join(&joied_str)?));

    // todo!("2. redirect to consent screen with flow attached to user session and scopes");
}

#[utoipa::path(
    post,
    path = "/authorize",
    responses(
        (status = 200, description = "current session info"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn finish_authorization(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<AuthorizationFinishRequest>,
) -> Result<Response, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let authorization_id = get_pending_authorization_id_from_cookies(&cookies, &global.settings)
        .ok_or_else(|| ApiErrorCodes::InvalidCode)?;

    let Ok(Some(pending_authorization)) =
        OauthPendingAuthorization::find_by_id(authorization_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    if !pending_authorization_checks(&pending_authorization, &auth, request.client_id) {
        return Err(ApiErrorCodes::InvalidCode);
    }

    let Ok(Some(client)) =
        OauthApplication::find_by_id(pending_authorization.client_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    let Ok(Some(mut authorization)) =
        OauthAuthorization::find_by_user_and_client_id(auth.user_id(), client.id, &global.database)
            .await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    // re-sanitize scopes just innnn case :)
    let updated_scopes = Scopes::from_bits(pending_authorization.requested_scopes)
        .sanitize(Scopes::from_bits(client.scopes));
    authorization.scopes = updated_scopes.bits();

    let pending_token = OauthPendingToken::builder()
        .client_id(client.id)
        .user_id(auth.user_id())
        .nonce(pending_authorization.nonce)
        .state(pending_authorization.state)
        .code_challenge(pending_authorization.code_challenge)
        .scopes(updated_scopes.bits())
        .build();

    let mut tx = global.database.begin().await.unwrap();
    authorization.update(&mut tx).await.unwrap();
    pending_token.insert(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    //  nope UWRAP evyrething now
    let mut url = Url::parse(&client.redirect_uri).unwrap();
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("code", &pending_token.code);
        if let Some(state) = pending_token.state {
            q.append_pair("state", &state);
        }
        q.append_pair("iss", global.settings.http.origin.as_str());
    }
    return Ok(redirect_to(&url));

    // todo!("redirect to client redirect_url")
}

#[axum::debug_handler]
#[utoipa::path(
    post,
    path = "/token",
    responses(
        (status = 200, description = "current session info"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn token(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, OauthError> {
    if request.grant_type != GrantType::AuthorizationCode {
        return Err(OauthError::new(
            OauthErrorCodes::UnsupportedGrantType,
            &global.settings.http.origin.clone(),
            &None,
        ));
        // todo!("err with UnsupportedGrantType")
    }

    let Ok(Some(client)) = OauthApplication::find_by_id(request.client_id, &global.database).await
    else {
        return Err(OauthError::new(
            OauthErrorCodes::InvalidClient,
            &global.settings.http.origin.clone(),
            &None,
        ));

        // todo!("err with InvalidClient")
    };

    let mut tx = global.database.begin().await.unwrap();
    let Ok(Some(pending_token)) = OauthPendingToken::take_by_id(request.code, &mut tx).await else {
        return Err(OauthError::new(
            OauthErrorCodes::InvalidClient,
            &global.settings.http.origin.clone(),
            &None,
        ));

        // todo!("err with InvalidClient")
    };

    if pending_token.client_id != client.id {
        return Err(OauthError::new(
            OauthErrorCodes::InvalidClient,
            &global.settings.http.origin.clone(),
            &None,
        ));

        // todo!("err with InvalidClient")
    }

    let redirect_uri = Url::parse(&client.redirect_uri).unwrap();

    if let Some(this_uri) = request.redirect_uri {
        let request_parsed = Url::parse(&this_uri);

        if request_parsed.is_err() {
            return Err(OauthError::new(
                OauthErrorCodes::InvalidRequest,
                &global.settings.http.origin.clone(),
                &None,
            ));

            // todo!("err with InvalidRequest")
        }
        let request_parsed = request_parsed.unwrap();

        if !valid_redirect_uri(&request_parsed, &redirect_uri) {
            return Err(OauthError::new(
                OauthErrorCodes::InvalidRequest,
                &global.settings.http.origin.clone(),
                &None,
            ));

            // todo!("err with InvalidRequest")
        }
    }

    let hashed_challenge =
        BASE64URL_NOPAD.encode(Sha256::digest(request.code_verifier.as_bytes()).as_bytes());

    if hashed_challenge != pending_token.code_challenge {
        return Err(OauthError::new(
            OauthErrorCodes::InvalidGrant,
            &global.settings.http.origin.clone(),
            &None,
        ));

        // todo!("err with InvalidGrant")
    }

    if !verify_client_secret(&request.client_secret, &client.secret) {
        return Err(OauthError::new(
            OauthErrorCodes::InvalidClient,
            &global.settings.http.origin.clone(),
            &None,
        ));

        // todo!("err with InvalidClient")
    }

    let token = get_hashed_secret();
    // re-re-sanitize scopes just innnn case :)
    let scopes = Scopes::from_bits(pending_token.scopes).sanitize(Scopes::from_bits(client.scopes));

    let oauth_token = OauthToken::builder()
        .client_id(client.id)
        .token(token.hash)
        .user_id(pending_token.user_id)
        .scopes(scopes.bits())
        .build();

    OauthToken::delete_all_by_user_and_client_id(pending_token.user_id, client.id, &mut tx)
        .await
        .unwrap();
    oauth_token.insert(&mut tx).await.unwrap();
    tx.commit().await.unwrap();

    Ok(Json(TokenResponse {
        access_token: token.code,
        token_type: "Bearer".to_string(),
        expires_in: chrono::Duration::MAX.num_seconds() as u64, // no expiry :)
        scope: scopes.to_string(),
        id_token: None, // id tokens for oidc... holy crap dude I actually don't have enough time to implement this
    }))
}

//oidc stuff please
