mod audit_log;
mod oauth;
mod session;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::{RE_AUTH_FLOW_LOGIN, flags::UserFlags},
    database::{id::UlidId, models::user::User as DbUser},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        extractor::Json,
        v1::types::{AlrightResponse, IdParam, ListDataRequest, ListDataResponse, User},
        validator::Valid,
    },
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(admin_list_users))
        .routes(routes!(admin_info_user))
        .routes(routes!(admin_edit_user))
        .nest("/{id}/session", session::routes())
        .nest("/{id}/oauth", oauth::routes())
        .nest("/{id}/audit", audit_log::routes())
}

/// List registered users
#[utoipa::path(
    get,
    params(ListDataRequest),
    path = "/list",
    tags = ["admin"],
    responses(
        (status = 200, description = "list of users", body = ListDataResponse<User>),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_list_users(
    State(global): State<Arc<GlobalState>>,
    Query(request): Query<ListDataRequest>,
) -> Result<Json<ListDataResponse<User>>, ApiErrorCodes> {
    let Ok(paginated) = DbUser::find_many_paginated(
        request.from,
        request.want_total.unwrap_or_default(),
        &global.database,
    )
    .await
    else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    let data: Vec<_> = paginated.items.into_iter().map(User::from).collect();

    Ok(Json(ListDataResponse {
        data,
        total: paginated.total_rows,
        next: paginated.next_id,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct UserUpdateRequest {
    #[validate( // mr fmt doesnt format this aberration.
        length(min = 3, max = 63, message = "must be between 4 letters and 64"), // counts from 0 duh
        regex(path = *RE_AUTH_FLOW_LOGIN, message = "must be alphanumeric and can contain underscores")
    )]
    pub login: Option<String>,
    #[validate(custom(function = "crate::auth::valid_email"))]
    pub email: Option<String>,
    pub flags: Option<i64>,
}

/// Get info about a user
#[utoipa::path(
    get,
    path = "/{id}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "info about an user", body = User),
        (status = 404, description = "user not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_info_user(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
) -> Result<Json<User>, ApiErrorCodes> {
    let Ok(Some(user)) = DbUser::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::DataNotFound("user"));
    };

    Ok(Json(user.into()))
}

/// Update an existing user
#[utoipa::path(
    patch,
    path = "/{id}",
    tags = ["admin"],
    params(
        ("id" = UlidId, description = "the id of the user"),
    ),
    responses(
        (status = 200, description = "successfully updated the user", body = AlrightResponse),
        (status = 404, description = "user not found", body = ApiError),
        (status = 403, description = "current user not allowed to do action", body = ApiError),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn admin_edit_user(
    State(global): State<Arc<GlobalState>>,
    Path(request): Path<IdParam<UlidId>>,
    Valid(Json(data)): Valid<Json<UserUpdateRequest>>,
) -> Result<Json<AlrightResponse>, ApiErrorCodes> {
    let Ok(Some(mut user)) = DbUser::find_by_id(request.id, &global.database).await else {
        return Err(ApiErrorCodes::DataNotFound("user"));
    };

    if let Some(login) = data.login {
        let Ok(None) = DbUser::find_by_login(login.clone(), &global.database).await else {
            return Err(ApiErrorCodes::LoginAlreadyAssociated);
        };

        user.login = login;
    }

    if let Some(email) = data.email {
        let Ok(None) = DbUser::find_by_email(email.clone(), &global.database).await else {
            return Err(ApiErrorCodes::EmailAlreadyAssociated);
        };

        user.email = email;
    }

    if let Some(flags) = data.flags {
        let flags = UserFlags::from_bits(flags);

        user.flags = flags.bits();
    }

    let mut tx = global.database.begin().await?;
    user.update(&mut tx).await?;
    tx.commit().await?;

    Ok(Json(AlrightResponse::default()))
}
