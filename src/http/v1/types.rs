use axum::response::IntoResponse;
use webauthn_rs::prelude::RegisterPublicKeyCredential;
use webauthn_rs_proto::PublicKeyCredential;

use crate::database::{
    self,
    id::UlidId,
    models::{
        user::UserId,
        user_session::PIDUserSessionId,
        user_webauthn::{PIDUserWebauthnId, UserWebauthn},
    },
};

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::user::User> for User {
    fn from(value: database::models::user::User) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct Session {
    pub id: PIDUserSessionId,
    pub active_expires_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::user_session::UserSession> for Session {
    fn from(value: database::models::user_session::UserSession) -> Self {
        Self {
            id: value.pid,
            active_expires_at: value.active_expires_at,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

// Copy and paste from webauthn_rs_proto::RegisterPublicKeyCredential with some fields being serde_json::Value. its for the utoipa stuff :)
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct RegisterPasskeyRequest {
    /// The id of the PublicKey credential, likely in base64.
    pub id: String,
    /// The id of the credential, as binary.
    #[serde(rename = "rawId")]
    pub raw_id: serde_json::Value,
    /// <https://w3c.github.io/webauthn/#dom-publickeycredential-response>
    pub response: serde_json::Value,
    /// The type of credential.
    #[serde(rename = "type")]
    pub type_: String,
    /// Unsigned Client processed extensions.
    #[serde(default, alias = "clientExtensionResults", alias = "extensions")]
    pub extensions: serde_json::Value,
}

impl TryFrom<RegisterPasskeyRequest> for RegisterPublicKeyCredential {
    type Error = serde_json::Error;

    fn try_from(value: RegisterPasskeyRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(serde_json::to_value(value)?)
    }
}

// Copy and paste from webauthn_rs_proto::PublicKeyCredential with some fields being serde_json::Value. its for the utoipa stuff :)
#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
pub struct AuthenticationPasskeyRequest {
    /// The credential Id, likely base64
    pub id: String,
    /// The binary of the credential id.
    #[serde(rename = "rawId")]
    pub raw_id: serde_json::Value,
    /// The authenticator response.
    pub response: serde_json::Value,
    /// Unsigned Client processed extensions.
    #[serde(default, alias = "clientExtensionResults")]
    pub extensions: serde_json::Value,
    /// The authenticator type.
    #[serde(rename = "type")]
    pub type_: String,
}

impl TryFrom<AuthenticationPasskeyRequest> for PublicKeyCredential {
    type Error = serde_json::Error;

    fn try_from(value: AuthenticationPasskeyRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(serde_json::to_value(value)?)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Otp,
    Totp,
    Passkey,
}

#[derive(
    Debug, serde::Deserialize, serde::Serialize, utoipa::ToSchema, smart_default::SmartDefault,
)]
pub struct AlrightResponse {
    #[default = true]
    ok: bool,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct Passkey {
    pub id: PIDUserWebauthnId,
    pub display_name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aaguid: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<UserWebauthn> for Passkey {
    fn from(value: UserWebauthn) -> Self {
        Self {
            id: value.pid,
            display_name: value.display_name,
            enabled: value.enabled,
            aaguid: value.aaguid,
            disabled_at: value.disabled_at,
            last_used_at: value.last_used_at,
            created_at: value.created_at,
        }
    }
}
pub enum RouteEither<L, R> {
    Left(L),
    Right(R),
}

impl<L: IntoResponse, R: IntoResponse> IntoResponse for RouteEither<L, R> {
    fn into_response(self) -> axum::response::Response {
        match self {
            RouteEither::Left(v) => v.into_response(),
            RouteEither::Right(v) => v.into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct AuditLog {
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::audit_log::AuditLog> for AuditLog {
    fn from(value: database::models::audit_log::AuditLog) -> Self {
        Self {
            action: value.action,
            metadata: value.metadata,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OauthApplication {
    pub id: UlidId,
    pub name: String,
    pub redirect_uri: String,
    pub public: bool,
    pub scopes: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::oauth_application::OauthApplication> for OauthApplication {
    fn from(value: database::models::oauth_application::OauthApplication) -> Self {
        Self {
            id: value.id,
            name: value.name,
            redirect_uri: value.redirect_uri,
            public: value.public,
            scopes: value.scopes,
            updated_at: value.updated_at,
            created_at: value.created_at,
        }
    }
}
