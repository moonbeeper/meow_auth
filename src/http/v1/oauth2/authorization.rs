use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Form, State},
};
use tower_cookies::Cookies;
use url::Url;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
    auth::flags::UserFlag,
    database::models::{
        oauth_application::OauthApplication, oauth_authorization::OauthAuthorization,
        oauth_pending_authorization::OauthPendingAuthorization,
        oauth_pending_token::OauthPendingToken,
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json as MJson,
        middleware::auth_manager::AuthContext,
        v1::types::{AlrightResponse, RouteEither},
    },
    oauth::{
        cookies::get_oauth_cookie,
        error::OauthErrorCodes,
        helpers::{action_new_authorization, action_past_authorized},
        pending_authorization_checks,
        response::{OAUTH_ISSUER, OauthResponse},
        scopes::{Scope, Scopes},
        types::{
            AuthorizationDecisionRequest, AuthorizationRequest, CodeChallengeMethod, PromptType,
            ResponseType,
        },
        valid_redirect_uri,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(finish_authorization))
        .routes(routes!(authorize))
}

/// Authorize an oauth application.
///
/// This endpoint is used to authorize an oauth application.
/// It will redirect to the consent screen if the current user has not authorized this application
/// else it will redirect to the application's redirect_uri with typical oauth stuff.
#[utoipa::path(
    get,
    path = "/authorize",
    tags = ["oauth_srv"],
    responses(
        (status = 303, description = "redirect to consent screen or redirect_uri with code"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn authorize(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Form(request): Form<AuthorizationRequest>,
) -> OauthResponse {
    OauthResponse::set_issuer(global.settings.http.origin.clone());

    if request.response_type != ResponseType::Code {
        return OauthResponse::new().error(
            OauthErrorCodes::UnsupportedResponseType,
            None,
            request.state.clone(),
        );
    }

    if request.code_challenge_method != CodeChallengeMethod::S256 {
        return OauthResponse::new().error(
            OauthErrorCodes::InvalidRequest,
            Some("code_challenge_method must be S256"),
            request.state.clone(),
        );
    }

    let Ok(Some(client)) = OauthApplication::find_by_id(request.client_id, &global.database).await
    else {
        return OauthResponse::new().error(
            OauthErrorCodes::InvalidClient,
            None,
            request.state.clone(),
        );
    };

    if client.disabled {
        return OauthResponse::new().error(
            OauthErrorCodes::InvalidClient,
            Some("client is disabled"),
            request.state.clone(),
        );
    }

    // client redirect_uri should always be valid.
    let mut redirect_url = Url::parse(&client.redirect_uri).unwrap();

    // swap redirect_url for provided one IF its valid and matches the client one.
    if let Some(this_uri) = request.redirect_uri.clone() {
        let Ok(requested_url) = Url::parse(&this_uri) else {
            return OauthResponse::new().error(
                OauthErrorCodes::InvalidRequest,
                Some("redirect_uri is not a valid url"),
                request.state.clone(),
            );
        };

        if !valid_redirect_uri(&requested_url, &redirect_url) {
            return OauthResponse::new().error(
                OauthErrorCodes::InvalidRedirect,
                None,
                request.state.clone(),
            );
        }

        redirect_url = requested_url;
    }

    let mut requested_scopes = Scopes::from_bits(client.scopes); // max client scopes

    if let Some(scopes_str) = request.scope.clone() {
        let requested = Scopes::from_str(&scopes_str);

        // if requesting too many scopes stop
        if !requested_scopes.contains(requested) {
            return OauthResponse::new().error(
                OauthErrorCodes::InvalidScope,
                None, // custom message?
                request.state.clone(),
            );
        }

        requested_scopes = requested;
    }

    let is_openid = requested_scopes.has(Scope::OpenId);

    if !auth.is_authenticated() {
        if is_openid && request.prompt == Some(PromptType::None) {
            return OauthResponse::new().error(
                OauthErrorCodes::LoginRequired,
                Some("this request needs an authenticated user"),
                request.state.clone(),
            );
        }

        return OauthResponse::new().error(
            OauthErrorCodes::AccessDenied,
            Some("this request needs an authenticated user"),
            request.state.clone(),
        );
    }

    if auth
        .user_flags()
        .has(UserFlag::CannotAuthorizeOauthApplications)
    {
        return OauthResponse::new().error(
            OauthErrorCodes::AccessDenied,
            Some("the requested action is not allowed for this account"),
            request.state.clone(),
        );
    }

    let Ok(authorization) =
        OauthAuthorization::find_by_user_and_client_id(auth.user_id(), client.id, &global.database)
            .await
    else {
        return OauthResponse::new().error(
            OauthErrorCodes::ServerError,
            None,
            request.state.clone(),
        );
    };

    if authorization.is_none() && request.prompt == Some(PromptType::None) {
        return OauthResponse::new().error(
            OauthErrorCodes::ConsentRequired,
            None,
            request.state.clone(),
        );
    }

    // at this point, we've validated the request, and we have an authenticated user and a correct client.
    // NOW we have to check if the user has already authorized this client with the requested scopes. If not,
    // we must create a pending authorization BUT not delete the current one and then redirect to the consent screen.
    // If accepted, we do the obvious, delete the current one and create a new one with the requested scopes and do
    // the normal flow.
    //
    // If they have already authorized the client with the requested scopes, we create a delete all past tokens and create
    // a new pending token for the client and then redirect to the redirect_uri with the code and state.

    // OH GOD. WHAT HAVE I DONE... god forgive me. I TRIED MY BEST TO MAKE IT
    // EASIER AND it ended half easier now BUT GOD IT IS UGLY LIKEEEEEEA AAAAGH
    let mut tx = match global.database.begin().await {
        Ok(v) => v,
        Err(_) => {
            return OauthResponse::new().error(
                OauthErrorCodes::ServerError,
                None,
                request.state.clone(),
            );
        }
    };

    // this is the part where i should make a uhh joke? idk man i ran out of parts to go.
    let going_to;
    if let Some(authorization) = authorization {
        going_to = match action_past_authorized(
            &request,
            client,
            authorization,
            requested_scopes,
            redirect_url,
            is_openid,
            &auth,
            &cookies,
            &global.settings,
            &mut tx,
        )
        .await
        {
            Err(_) => {
                return OauthResponse::new().error(
                    OauthErrorCodes::ServerError,
                    None,
                    request.state.clone(),
                );
            }
            Ok(v) => v,
        };
    } else {
        going_to = match action_new_authorization(
            &request,
            client,
            requested_scopes,
            redirect_url,
            is_openid,
            None,
            &auth,
            &cookies,
            &global.settings,
            &mut tx,
        )
        .await
        {
            Err(_) => {
                return OauthResponse::new().error(
                    OauthErrorCodes::ServerError,
                    None,
                    request.state.clone(),
                );
            }
            Ok(v) => v,
        };
    }

    match tx.commit().await {
        Ok(v) => v,
        Err(_) => {
            return OauthResponse::new().error(
                OauthErrorCodes::ServerError,
                None,
                request.state.clone(),
            );
        }
    };

    OauthResponse::new().redirect(going_to.to_string())
}

/// Give consent to a pending oauth authorization.
#[utoipa::path(
    post,
    path = "/consent",
    tags = ["oauth_srv"],
    responses(
        (status = 303, description = "consent given, redirected to client's redirect url"),
        (status = 200, description = "consent denied"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn finish_authorization(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(cookies): Extension<Cookies>,
    Json(request): Json<AuthorizationDecisionRequest>,
) -> Result<RouteEither<OauthResponse, MJson<AlrightResponse>>, ApiErrorCodes> {
    OauthResponse::set_issuer(global.settings.http.origin.clone());
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let authorization_id =
        get_oauth_cookie(&cookies, &global.settings).ok_or_else(|| ApiErrorCodes::FlowNotFound)?;

    let Ok(Some(pending_authorization)) =
        OauthPendingAuthorization::find_by_id(authorization_id, &global.database).await
    else {
        return Err(ApiErrorCodes::FlowNotFound);
    };

    if !pending_authorization_checks(&pending_authorization, &auth, request.client_id) {
        return Err(ApiErrorCodes::FlowNotFound);
    }

    if !request.consent {
        let mut tx = global.database.begin().await?;
        pending_authorization.delete_all(&mut tx).await?;
        tx.commit().await?;

        return Ok(RouteEither::Right(MJson(AlrightResponse::default())));
    }

    let Ok(Some(oauth_client)) =
        OauthApplication::find_by_id(pending_authorization.client_id, &global.database).await
    else {
        return Err(ApiErrorCodes::InvalidCode);
    };

    // re-sanitize scopes just in case Oauth app owner has updated them for some goddamn reason
    let requested_scopes = Scopes::from_bits(pending_authorization.requested_scopes)
        .sanitize(Scopes::from_bits(oauth_client.scopes));

    let pending_token = OauthPendingToken::builder()
        .client_id(oauth_client.id)
        .code_challenge(pending_authorization.code_challenge.clone())
        .nonce(pending_authorization.nonce.clone())
        .scopes(requested_scopes.bits())
        .state(pending_authorization.state.clone())
        .user_id(auth.user_id())
        .is_openid(pending_authorization.is_openid)
        .build();

    let new_authorization = OauthAuthorization::builder()
        .user_id(auth.user_id())
        .client_id(oauth_client.id)
        .scopes(requested_scopes.bits())
        .build();

    let mut tx = global.database.begin().await.unwrap();
    let mut audit_as_update = false;
    if let Some(old_auth_id) = pending_authorization.old_authorization_id {
        OauthAuthorization::delete_by_id(old_auth_id, &mut tx).await?;
        audit_as_update = true;
    }
    pending_authorization.delete_all(&mut tx).await?;
    new_authorization.insert(&mut tx).await?;

    pending_token.delete_all(&mut tx).await?;
    pending_token.insert(&mut tx).await?;

    audit::log(
        auth.user_id(),
        auth.user_id(),
        if audit_as_update {
            AuditAction::OauthAuthorizationUpdated
        } else {
            AuditAction::OauthAuthorizationApproved
        },
        None,
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    // redirect_uri must be good to get to this point.
    let mut url = Url::parse(&pending_authorization.redirect_url).unwrap();
    {
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("code", &pending_token.code);
        query_pairs.append_pair("iss", OAUTH_ISSUER.get().unwrap().as_ref());
        if let Some(state) = pending_authorization.state {
            query_pairs.append_pair("state", &state);
        }
    }

    Ok(RouteEither::Left(
        OauthResponse::new().redirect(url.to_string()),
    ))
}
