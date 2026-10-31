use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{
        AuditAction, AuditEntry, ResourceType,
        actor::AuditActor,
        changes::{AuditChange, AuditDiff, AuditField},
    },
    database::{
        id::UlidId,
        models::{
            oauth_application::OauthApplication as DbOauthApplication,
            oauth_authorization::OauthAuthorization as DbOauthAuthorization,
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
            AlrightResponse, IdParam, ListDataRequest, ListDataResponse, OauthApplication,
            TwoIdParam,
        },
        validator::Valid,
    },
    oauth::{scope::Scopes, secrets::get_secret_pair},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(admin_list_user_applications))
        .routes(routes!(admin_edit_user_application))
        .routes(routes!(admin_delete_user_application))
        .routes(routes!(admin_rotate_secret_user_application))
        .routes(routes!(admin_revoke_all_authorizations_application))
}

/// List oauth applications of a user
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "list of oauth applications", body = ListDataResponse<OauthApplication>),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_list_user_applications(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
    Query(data): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<OauthApplication>>, ApiErrorCodes> {
    let Ok(paginated) = DbOauthApplication::find_many_by_user_id_paginated(
        request.id,
        data.from,
        data.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let data: Vec<_> = paginated
        .items
        .into_iter()
        .map(OauthApplication::from)
        .collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct OauthApplicationData {
    #[validate(length(
        min = 4,
        max = 32,
        message = "name must be between 4 and 32 characters"
    ))]
    pub name: Option<String>,
    #[validate(url(message = "must be a valid url"))]
    pub redirect_uri: Option<String>,
    pub public: Option<bool>,
    pub disabled: Option<bool>,
    pub scopes: Option<String>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OauthApplicationDataResponse {
    pub id: UlidId,
    pub secret: String,
}

/// Update an existing oauth application
#[utoipa::path(
    patch,
    path = "/{cid}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the oauth application")
    ),
    responses(
        (status = 200, description = "successfully updated the oauth application", body = AlrightResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_edit_user_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
    Valid(Json(data)): Valid<Json<OauthApplicationData>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) =
        DbOauthApplication::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let before = app.clone();

    if let Some(disabled) = data.disabled {
        app.disabled = disabled;
    }

    if let Some(scopes) = data.scopes {
        let scopes = Scopes::from_str(&scopes, &global.oauth_scope_registry);
        app.scopes = scopes.to_string();
    }

    if let Some(name) = data.name {
        app.name = name;
    }

    if let Some(redirect_uri) = data.redirect_uri {
        app.redirect_uri = redirect_uri;
    }

    if let Some(public) = data.public {
        app.public = public;
    }

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;

    AuditEntry::builder()
        .user_id(app.user_id)
        .actor(AuditActor::Admin(auth.user_id()))
        .action(AuditAction::OauthApplicationUpdated)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .resource_type(Some(ResourceType::OauthApplication))
        .resource_id(Some(app.id))
        .changes(app.audit_changes(&before))
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Delete an existing oauth application
#[utoipa::path(
    delete,
    path = "/{cid}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the oauth application")
    ),
    responses(
        (status = 200, description = "successfully deleted the oauth application"),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_delete_user_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthApplication::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let mut tx = global.database.begin().await?;
    app.delete(&mut tx).await?;

    AuditEntry::builder()
        .user_id(app.user_id)
        .actor(AuditActor::Admin(auth.user_id()))
        .action(AuditAction::OauthApplicationDeleted)
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .resource_type(Some(ResourceType::OauthApplication))
        .resource_id(Some(app.id))
        .changes(vec![
            AuditChange::deleted(AuditField::DisplayName, &app.name),
            AuditChange::deleted(AuditField::RedirectUri, &app.redirect_uri),
            AuditChange::deleted(AuditField::Public, app.public),
            AuditChange::deleted(AuditField::Scopes, &app.scopes),
            AuditChange::deleted(AuditField::Enabled, !app.disabled),
        ])
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}

/// Rotate the secret of an existing oauth application
#[utoipa::path(
    patch,
    path = "/{cid}/rotate_keys",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the oauth application")
    ),
    responses(
        (status = 200, description = "successfully rotated the oauth application secret", body = OauthApplicationDataResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_rotate_secret_user_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
) -> Result<Json<OauthApplicationDataResponse>, ApiErrorCodes> {
    let Ok(Some(mut app)) =
        DbOauthApplication::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let keys = get_secret_pair(&global.settings);
    app.secret = keys.hash_bytes;

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;

    AuditEntry::builder()
        .user_id(app.user_id)
        .actor(AuditActor::Admin(auth.user_id()))
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .action(AuditAction::OauthApplicationKeysRotated)
        .resource_type(Some(ResourceType::OauthApplication))
        .resource_id(Some(app.id))
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: keys.secret,
    }))
}

// TODO: Job to revoke all authorizations AND add an audit log for each user. Not only for the owner duh.
/// Revoke all authorizations of an existing oauth application
#[utoipa::path(
    post,
    path = "/{cid}/revoke_authorizations",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the oauth application")
    ),
    responses(
        (status = 200, description = "successfully revoked all oauth application authorizations", body = OauthApplicationDataResponse),
        (status = 404, description = "oauth application not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_revoke_all_authorizations_application(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Extension(ip_ctx): Extension<IpContext>,
    Extension(user_agent): Extension<UserAgentContext>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthApplication::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    };

    if app.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("oauth application"));
    }

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;
    let before = DbOauthAuthorization::count_by_app_id(app.id, &global.database)
        .await?
        .unwrap_or_default();
    let after = DbOauthAuthorization::delete_all_by_client_id(app.id, &mut tx).await?;

    AuditEntry::builder()
        .user_id(app.user_id)
        .actor(AuditActor::Admin(auth.user_id()))
        .actor_ip(ip_ctx.ip_addr())
        .actor_location(ip_ctx.location())
        .actor_user_agent(user_agent.agent())
        .action(AuditAction::OauthAuthorizationsRevoked)
        .resource_type(Some(ResourceType::OauthApplication))
        .resource_id(Some(app.id))
        .changes(vec![AuditChange::change(
            AuditField::Count,
            before,
            before.cast_unsigned().saturating_sub(after),
        )])
        .build()
        .save(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
