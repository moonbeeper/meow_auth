use std::str::FromStr;

use axum::response::IntoResponse;
use serde::Deserialize;
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
    pub flags: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::user::User> for User {
    fn from(value: database::models::user::User) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            flags: value.flags,
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
    /// The Authenticator Attestation GUID. An identifier used to determine the type of authenticator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aaguid: Option<uuid::Uuid>,
    /// The point in time that the passkey was permanently disabled.
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
    pub id: UlidId,
    pub action: String,
    pub user_id: UlidId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<UlidId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_login: Option<String>,
    pub was_self: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::audit_log::AuditLogLogin> for AuditLog {
    fn from(value: database::models::audit_log::AuditLogLogin) -> Self {
        Self {
            id: value.id,
            action: value.action,
            user_id: value.user_id,
            user_login: Some(value.user_login),
            actor_id: Some(value.actor_id),
            actor_login: Some(value.actor_login),
            was_self: value.user_id == value.actor_id,
            metadata: value.metadata,
            created_at: value.created_at,
        }
    }
}

impl From<database::models::audit_log::AuditLog> for AuditLog {
    fn from(value: database::models::audit_log::AuditLog) -> Self {
        Self {
            id: value.id,
            action: value.action,
            user_id: value.user_id,
            user_login: None,
            actor_id: None,
            actor_login: None,
            was_self: value.user_id == value.actor_id,
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct OauthAuthorization {
    pub id: UlidId,
    pub name: String,
    pub redirect_uri: String,
    pub scopes: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ListDataResponse<T> {
    pub data: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // fixes "[Circular]" in openapi schema
    #[schema(value_type = Option<String>, format = Ulid)]
    pub next: Option<UlidId>,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub struct ListDataRequest {
    /// The id of the last item from the previous page. Normally provided in the `next` field of the previous response.
    #[serde(
        default,
        deserialize_with = "please_shut_up_and_let_me_use_an_empty_string_as_none"
    )]
    pub from: Option<UlidId>,
    /// If the total number of items should be returned.
    #[serde(
        default,
        deserialize_with = "please_shut_up_and_let_me_use_an_empty_string_as_none"
    )]
    pub want_total: Option<bool>,
}

fn please_shut_up_and_let_me_use_an_empty_string_as_none<'a, T, A>(
    s: T,
) -> Result<Option<A>, T::Error>
where
    T: serde::Deserializer<'a>,
    A: FromStr,
    A::Err: std::fmt::Display,
{
    let s = String::deserialize(s)?;
    if s.is_empty() {
        return Ok(None);
    }
    s.parse::<A>().map(Some).map_err(serde::de::Error::custom)
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct IdParam<T> {
    pub id: T,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
pub struct TwoIdParam<T, G> {
    pub id: T,
    #[serde(alias = "cid")]
    pub child_id: G,
}
