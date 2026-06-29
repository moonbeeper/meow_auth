use std::sync::Arc;

use axum::extract::{Path, Query, State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    database::{id::UlidId, models::oauth_application::OauthApplication as DbOauthApplication},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::types::{
            AlrightResponse, IdParam, ListDataRequest, ListDataResponse, OauthApplication,
            TwoIdParam,
        },
        validator::Valid,
    },
    oauth::{scopes::Scopes, secrets::get_secret_pair},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(admin_list_user_applications))
        .routes(routes!(admin_edit_user_application))
        .routes(routes!(admin_delete_user_application))
        .routes(routes!(admin_rotate_secret_user_application))
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
    pub name: String,
    #[validate(url(message = "must be a valid url"))]
    pub redirect_uri: String,
    pub public: bool,
    pub scopes: i64,
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

    let scopes = Scopes::from_bits(data.scopes).sanitize(Scopes::all());
    app.name = data.name;
    app.redirect_uri = data.redirect_uri;
    app.public = data.public;
    app.scopes = scopes.bits();

    let mut tx = global.database.begin().await?;
    app.update(&mut tx).await?;
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
    tx.commit().await?;

    Ok(Json(OauthApplicationDataResponse {
        id: app.id,
        secret: keys.secret,
    }))
}
