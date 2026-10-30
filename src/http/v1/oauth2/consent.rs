use std::sync::Arc;

use axum::{Extension, extract::State};
use tower_cookies::Cookies;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::{
        oauth_application::OauthApplication, oauth_pending_authorization::OauthPendingAuthorization,
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
    },
    oauth::{
        cookies::get_oauth_cookie, response::OauthResponse, scope::Scopes, types::ConsentMetadata,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new().routes(routes!(oauth_consent_info))
}

// TODO: should match the userid with the user session id!

/// Get the pending oauth authorization metadata
#[utoipa::path(
    get,
    path = "/consent",
    tags = ["oauth_srv"],
    responses(
        (status = 200, description = "current pending authorization flow consent info", body = ConsentMetadata),
        (status = 404, description = "pending authorization flow not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn oauth_consent_info(
    State(global): State<Arc<GlobalState>>,
    Extension(cookies): Extension<Cookies>,
) -> Result<Json<ConsentMetadata>, ApiErrorCodes> {
    OauthResponse::set_issuer(global.settings.http.origin.clone());
    let pending_id =
        get_oauth_cookie(&cookies, &global.settings).ok_or(ApiErrorCodes::FlowNotFound)?;

    let Ok(Some(pending_authorization)) =
        OauthPendingAuthorization::find_by_id(pending_id, &global.database).await
    else {
        // shouldnt delete cookies. Else a user refreshes becuase something got stuck and has to redo the flow,
        // WHEN THE FLOW IS STILL VALID. bad >:( decision stupid past moon.
        // also, get_oauth_cookie method had the delete cookie flag also set lol, so that deleting was useless
        return Err(ApiErrorCodes::FlowNotFound);
    };

    let Ok(Some(client)) =
        OauthApplication::find_by_id(pending_authorization.client_id, &global.database).await
    else {
        return Err(ApiErrorCodes::FlowNotFound);
    };

    let client_scopes = Scopes::from_str(&client.scopes, &global.oauth_scope_registry);

    let new_scopes = Scopes::from_str(
        &pending_authorization.requested_scopes,
        &global.oauth_scope_registry,
    )
    .sanitize(client_scopes.clone());

    let old_scopes = if let Some(old) = pending_authorization.old_scopes {
        Scopes::from_str(&old, &global.oauth_scope_registry).sanitize(client_scopes)
    } else {
        Scopes::default()
    };

    Ok(Json(ConsentMetadata {
        id: client.id,
        name: client.name,
        scopes: new_scopes.to_string(),
        old_scopes: old_scopes.to_string(),
        redirect_url: pending_authorization.redirect_url,
        created_at: client.created_at,
    }))
}
