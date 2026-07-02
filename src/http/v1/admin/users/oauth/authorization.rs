use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension,
    extract::{Path, Query, State},
};
use serde_json::json;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    audit::{self, AuditAction},
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
        middleware::auth_manager::AuthContext,
        v1::types::{
            AlrightResponse, IdParam, ListDataRequest, ListDataResponse, OauthApplication,
            OauthAuthorization, TwoIdParam,
        },
    },
    oauth::scopes::Scopes,
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(admin_list_user_oauth_authorizations))
        .routes(routes!(admin_delete_user_oauth_authorization))
}

/// List oauth authorizations of a user
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "list of oauth authorizations", body = ListDataResponse<OauthApplication>),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_list_user_oauth_authorizations(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
    Query(data): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<OauthAuthorization>>, ApiErrorCodes> {
    let Ok(paginated) = DbOauthAuthorization::find_many_by_user_id_paginated(
        request.id,
        data.from,
        data.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let client_ids: Vec<_> = paginated.items.iter().map(|v| v.client_id).collect();

    let Ok(apps) = DbOauthApplication::find_many_by_id(&client_ids, &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };
    let app_map: HashMap<_, _> = apps.into_iter().map(|v| (v.id, v)).collect();

    let data: Vec<_> = paginated
        .items
        .into_iter()
        .map(|v| {
            let app = &app_map[&v.client_id];
            let scopes = Scopes::from_bits(v.scopes).sanitize(Scopes::from_bits(app.scopes));
            OauthAuthorization {
                id: v.id,
                name: app.name.clone(),
                redirect_uri: app.redirect_uri.clone(),
                scopes: scopes.bits(),
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

/// Revoke an oauth authorization
#[utoipa::path(
    delete,
    path = "/{cid}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
        ("cid" = UlidId, description = "the id of the oauth authorization")
    ),
    responses(
        (status = 200, description = "successfully revoked the oauth authorization", body = AlrightResponse),
        (status = 404, description = "oauth authorization not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_delete_user_oauth_authorization(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Path(request): Path<TwoIdParam<UlidId, UlidId>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(app)) = DbOauthAuthorization::find_by_id(request.child_id, &global.database).await
    else {
        return Err(ApiErrorCodes::DataNotFound("oauth authorization"));
    };

    if app.user_id != request.id {
        return Err(ApiErrorCodes::DataNotFound("oauth authorization"));
    }

    let mut tx = global.database.begin().await?;
    app.delete(&mut tx).await?;
    audit::log(
        auth.user_id(),
        app.user_id,
        AuditAction::OauthAuthorizationsRevoked,
        Some(json!({
            "app_id": app.id.to_string(),
        })),
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
