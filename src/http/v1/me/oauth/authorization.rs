use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{
        AuditAction, AuditEntry, ResourceType,
        actor::AuditActor,
        changes::{AuditChange, AuditField},
    },
    database::{
        id::UlidId,
        models::{
            oauth_application::OauthApplication as DbOauthApplication,
            oauth_authorization::OauthAuthorization as DbOauthAuthorization,
            oauth_token::OauthToken,
        },
    },
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        middleware::{
            auth_manager::AuthContext, browser_agent_manager::UserAgentContext,
            ip_manager::IpContext,
        },
        v1::types::{
            AlrightResponse, ClientType, ListDataRequest, ListDataResponse, OauthAuthorization,
        },
    },
    oauth::scope::Scopes,
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(list_oauth_authorizations))
        .routes(routes!(revoke_oauth_authorization))
}

/// List your oauth authorizations
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["oauth"],
    responses(
        (status = 200, description = "list of oauth authorizations", body = ListDataResponse<OauthAuthorization>),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn list_oauth_authorizations(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Query(request): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<OauthAuthorization>>, ApiErrorCodes> {
    let Ok(paginated) = DbOauthAuthorization::find_many_by_user_id_paginated(
        auth.user_id(),
        request.from,
        request.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    // WOW, wasted time on not seeing that i set the value to the id of the authorization INSTEAD of the client.
    // GREAT BIRD BRAIN ME GREAT DUDE AGH
    let client_ids: Vec<_> = paginated.items.iter().map(|v| v.client_id).collect();

    let Ok(apps) = DbOauthApplication::find_many_by_id(&client_ids, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    // need Eq to be able to use HashSet... bu actually why? like i am just fetching
    // some specific rows... stupid me bird brain crap
    let app_map: HashMap<_, _> = apps.into_iter().map(|v| (v.id, v)).collect();

    let data: Vec<_> = paginated
        .items
        .into_iter()
        .map(|v| {
            let app = &app_map[&v.client_id];
            let scopes = Scopes::from_str(&v.scopes, &global.oauth_scope_registry)
                .sanitize(Scopes::from_str(&app.scopes, &global.oauth_scope_registry));
            OauthAuthorization {
                id: v.id,
                name: app.name.clone(),
                redirect_uri: app.redirect_uri.clone(),
                scopes: scopes.to_string(),
                client_type: ClientType::from(app.client_type),
                last_used_at: v.last_used_at,
                updated_at: v.updated_at,
                created_at: v.created_at,
            }
        })
        .collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate, utoipa::IntoParams)]
pub struct OauthAuthorizationIdParam {
    pub id: UlidId,
}

/// Revoke an oauth authorization
#[utoipa::path(
    delete,
    params(OauthAuthorizationIdParam),
    path = "/{id}",
    tags = ["oauth"],
    responses(
        (status = 200, description = "successfully revoked the oauth authorization", body = AlrightResponse),
        (status = 404, description = "oauth authorization not found", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn revoke_oauth_authorization(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<OauthAuthorizationIdParam>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthAuthorization::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::DataNotFound("oauth authorization"));
    };

    if app.user_id != auth.user_id() {
        return Err(ApiErrorCodes::DataNotFound("oauth authorization"));
    }

    let mut tx = global.database.begin().await?;
    app.delete(&mut tx).await?;
    OauthToken::delete_all_by_user_and_client_id(auth.user_id(), app.client_id, &mut tx).await?;

    AuditEntry::builder()
        .user_id(auth.user_id())
        .actor(AuditActor::UserSelf(auth.user_id()))
        .action(AuditAction::OauthAuthorizationRevoked)
        .resource_id(Some(app.id))
        .resource_type(Some(ResourceType::OauthAuthorization))
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .changes(vec![AuditChange::deleted(AuditField::Scopes, &app.scopes)])
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
