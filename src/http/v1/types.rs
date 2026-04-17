use base64urlsafedata::Base64UrlSafeData;
use webauthn_rs::prelude::RegisterPublicKeyCredential;
use webauthn_rs_proto::{
    AuthenticatorAttestationResponseRaw, CreationChallengeResponse,
    RegistrationExtensionsClientOutputs,
};

use crate::database::{
    self,
    models::{user::UserId, user_session::PIDUserSessionId},
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
