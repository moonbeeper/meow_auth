use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    build,
    global::GlobalState,
    http::{extractor::Json, v1::types::AlrightResponse},
};

pub fn routes() -> OpenApiRouter<Arc<GlobalState>> {
    OpenApiRouter::new()
        .routes(routes!(health))
        .routes(routes!(application_info))
}

/// Health endpoint, returns always OK
#[utoipa::path(
    get,
    path = "/ok",
    tags = ["application"],
    responses(
        (status = 200, description = "simple ok", body = AlrightResponse),
    )
)]
pub async fn health() -> Json<AlrightResponse> {
    Json(AlrightResponse::default())
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationBuildProfile {
    Debug,
    Release,
}

impl From<&str> for ApplicationBuildProfile {
    fn from(value: &str) -> Self {
        if value == "debug" {
            Self::Debug
        } else {
            Self::Release
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApplicationBuildInfo {
    profile: ApplicationBuildProfile,
    build_time: String,
    git_hash: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ApplicationInfoResponse {
    info: String,
    name: String,
    version: String,
    build: ApplicationBuildInfo,
}

/// Info about the application
#[utoipa::path(
    get,
    path = "/",
    tags = ["application"],
    responses(
        (status = 200, description = "application info", body = ApplicationInfoResponse),
    )
)]
pub async fn application_info() -> Json<ApplicationInfoResponse> {
    let build_info = ApplicationBuildInfo {
        profile: ApplicationBuildProfile::from(build::BUILD_RUST_CHANNEL),
        build_time: build::BUILD_TIME.to_string(),
        git_hash: build::COMMIT_HASH.to_string(),
    };

    let response = ApplicationInfoResponse {
        name: build::PROJECT_NAME.to_string(),
        info: "hi there... its working wahoo. could you leave already? I did my job".to_string(),
        version: build::PKG_VERSION.to_string(),
        build: build_info,
    };

    Json(response)
}
