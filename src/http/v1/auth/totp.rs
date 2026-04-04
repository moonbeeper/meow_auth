use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    auth::totp::{create_user_totp, decrypt_secrets, make_totp, usable_recovery_codes},
    database::models::{user::User, user_totp::UserTotp as DbUserTotp},
    global::GlobalState,
    http::{
        error::{ApiError, ApiErrorCodes},
        middleware::auth_manager::AuthContext,
    },
    job_queue::QueuedJob as _,
    mailer::{Email, MailerJob},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(create_totp))
        .routes(routes!(exchange_totp))
        .routes(routes!(disable_totp))
        .routes(routes!(recovery_codes_totp))
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CreateTotpResponse {
    uri: String,
    secret: String,
    recovery_codes: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "totp relevant info", body = CreateTotpResponse),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn create_totp(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
) -> Result<Json<CreateTotpResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if user.totp_enabled {
        return Err(ApiErrorCodes::TotpAlreadyEnabled);
    }

    let mut tx = global.database.begin().await?;
    DbUserTotp::delete_all_by_user(auth.user_id(), &mut tx).await?;
    tx.commit().await?;

    let totp = create_user_totp(auth.user_id(), &global.database, &global.settings)
        .await
        .map_err(|e| {
            tracing::error!("something went wrong while creating the user totp: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    let totp_client =
        make_totp(user.login, totp.secret.clone(), &global.settings).map_err(|e| {
            tracing::error!("something went wrong while creating the totp client: {e}");
            ApiErrorCodes::InternalServerError
        })?;
    let uri = totp_client.get_url();

    Ok(Json(CreateTotpResponse {
        uri,
        secret: totp.secret,
        recovery_codes: totp.recovery_codes,
    }))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct VerifyTotpRequest {
    code: String,
}

#[utoipa::path(
    post,
    path = "/exchange",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "totp successfully enabled"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn exchange_totp(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<(), ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if user.totp_enabled {
        return Err(ApiErrorCodes::TotpAlreadyEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpFlowNotFound);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client =
        make_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
            tracing::error!("something went wrong while creating the totp client: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::TotpInvalidCode);
    }

    let mut tx = global.database.begin().await?;
    user.totp_enabled = true;
    user.update(&mut tx).await?;
    db_totp.update(&mut tx).await?;
    tx.commit().await?;

    let email = Email::builder()
        .text(format!(
            "hi {}! your account now has totp enabled :)",
            user.login
        ))
        .to(user.email)
        .subject("totp enabled".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(())
}

#[utoipa::path(
    delete,
    path = "/",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "totp successfully disabled"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn disable_totp(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<(), ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(mut user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if !user.totp_enabled {
        return Err(ApiErrorCodes::TotpNotEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpNotEnabled);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client =
        make_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
            tracing::error!("something went wrong while creating the totp client: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::TotpInvalidCode);
    }

    let mut tx = global.database.begin().await?;
    user.totp_enabled = false;
    user.update(&mut tx).await?;
    db_totp.delete(&mut tx).await?;
    tx.commit().await?;

    let email = Email::builder()
        .text(format!(
            "hi {}! your account now has totp disabled :)",
            user.login
        ))
        .to(user.email)
        .subject("totp disabled".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(())
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RecoveryCodesTotpResponse {
    recovery_codes: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/recovery",
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, description = "usable totp recovery codes"),
        (status = 500, description = "internal server error", body = ApiError)
    )
)]
pub async fn recovery_codes_totp(
    State(global): State<Arc<GlobalState>>,
    Extension(auth): Extension<AuthContext>,
    Json(request): Json<VerifyTotpRequest>,
) -> Result<Json<RecoveryCodesTotpResponse>, ApiErrorCodes> {
    if !auth.is_authenticated() {
        return Err(ApiErrorCodes::Unauthenticated);
    }

    let Ok(Some(user)) = User::find_by_id(auth.user_id(), &global.database).await else {
        return Err(ApiErrorCodes::InternalServerError);
    };

    if !user.totp_enabled {
        return Err(ApiErrorCodes::TotpNotEnabled);
    }

    let Ok(Some(db_totp)) = DbUserTotp::find_one_by_user(auth.user_id(), &global.database).await
    else {
        return Err(ApiErrorCodes::TotpNotEnabled);
    };

    let encrypted_secrets = db_totp.clone().into();
    let totp = decrypt_secrets(&encrypted_secrets, &global.settings).map_err(|e| {
        tracing::error!("something went wrong while decrypting totp secrets: {e}");
        ApiErrorCodes::InternalServerError
    })?;
    let totp_client =
        make_totp(user.login.clone(), totp.secret, &global.settings).map_err(|e| {
            tracing::error!("something went wrong while creating the totp client: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    if !totp_client.check_current(&request.code).unwrap_or(false) {
        return Err(ApiErrorCodes::TotpInvalidCode);
    }

    let recovery_codes = usable_recovery_codes(&db_totp, &totp.recovery_secret);

    let mut tx = global.database.begin().await?;
    db_totp.update(&mut tx).await?;
    tx.commit().await?;

    let email = Email::builder()
        .text(format!(
            "hi {}! your totp recovery codes were just seen right now :)",
            user.login
        ))
        .to(user.email)
        .subject("totp recovery codes seen".to_string())
        .build();

    MailerJob::dispatch(&global.database, email)
        .await
        .map_err(|e| {
            tracing::error!("failed dispatching job: {e}");
            ApiErrorCodes::InternalServerError
        })?;

    Ok(Json(RecoveryCodesTotpResponse { recovery_codes }))
}
