use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::models::user::User,
    global::GlobalState,
    http::middleware::{oauth_manager::OauthContext, require_oauth::RequireOauthLayer},
    oauth::{
        error::OauthErrorCodes, openid::get_id_token_data, response::OauthResponse, scopes::Scope,
        types::OpenIdUserInfo,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(openid_userinfo))
        .layer(RequireOauthLayer::new())
}

/// Get the OpenId userinfo for the token's user.
///
/// This handler requires that the token has the `openid` scope.
#[utoipa::path(
    get,
    path = "/userinfo",
    tags = ["oauth_srv"],
    responses(
        (status = 200, description = "openid userinfo handler", body = OpenIdUserInfo),
    )
)]
pub async fn openid_userinfo(
    State(global): State<Arc<GlobalState>>,
    Extension(oauth): Extension<OauthContext>,
) -> Result<Json<OpenIdUserInfo>, OauthResponse> {
    if !oauth.scopes().has(Scope::OpenId) {
        return Err(OauthResponse::new().error(
            OauthErrorCodes::InsufficientScope,
            Some("missing openid scope"),
            None,
        ));
    }

    let Ok(Some(user)) = User::find_by_id(oauth.user_id(), &global.database).await else {
        return Err(OauthResponse::new().error(OauthErrorCodes::InvalidToken, None, None));
    };

    let user_info = get_id_token_data(
        user,
        oauth.client_id(),
        None,
        oauth.scopes(),
        &global.settings,
    );

    Ok(Json(user_info.into()))
}
