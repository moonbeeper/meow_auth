pub mod actor;
pub mod changes;

use std::{fmt::Display, net::IpAddr};

use sqlx::PgTransaction;

use crate::{
    audit::{actor::AuditActor, changes::AuditChange},
    database::{
        id::UlidId,
        models::{audit_log::AuditLog as DbAuditLog, user::UserId},
    },
};

pub const SYSTEM_USER_ID: UlidId = UlidId(ulid::Ulid::from_bytes(1u128.to_be_bytes()));

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AuditAction {
    SessionCreated, // should add metadata for what was used to create it (webauthn or otp)
    SessionRevoked,
    SessionsRevoked, // close all sessions
    EmailChangeRequested,
    EmailChanged,
    NameChanged,
    AccountCreated,
    // NOT IMPLEMENTED
    AccountDeleted, // TODO: THE HANDLER AAAGH GOD please dont incinerate me
    TotpEnabled,
    PasskeyAdded,
    PasskeyRemoved,
    PasskeyRenamed,
    PasskeyDisabled,
    TotpDisabled,
    TotpRecoveryCodeUsed,
    TotpRecoveryCodesSeen,
    SudoEnabled, // should add metadata for what was used to enable it (like the session)
    OauthApplicationCreated,
    OauthApplicationUpdated,
    OauthApplicationDeleted,
    OauthApplicationKeysRotated,
    OauthAuthorizationIntiated,
    OauthAuthorizationApproved,
    OauthAuthorizationDenied,
    OauthAuthorizationRevoked,
    OauthAuthorizationUpdated,
    OauthAuthorizationsRevoked,
    OauthTokenRevoked,
    UserUpdated,
}

impl Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::SessionCreated => write!(f, "session_created"),
            AuditAction::SessionRevoked => write!(f, "session_deleted"),
            AuditAction::SessionsRevoked => write!(f, "sessions_deleted"),
            AuditAction::EmailChangeRequested => write!(f, "email_change_requested"),
            AuditAction::EmailChanged => write!(f, "email_changed"),
            AuditAction::NameChanged => write!(f, "login_changed"),
            AuditAction::AccountCreated => write!(f, "account_created"),
            AuditAction::AccountDeleted => write!(f, "account_deleted"),
            AuditAction::TotpEnabled => write!(f, "totp_enabled"),
            AuditAction::PasskeyAdded => write!(f, "passkey_added"),
            AuditAction::PasskeyRemoved => write!(f, "passkey_removed"),
            AuditAction::PasskeyRenamed => write!(f, "passkey_renamed"),
            AuditAction::PasskeyDisabled => write!(f, "passkey_disabled"),
            AuditAction::TotpDisabled => write!(f, "totp_disabled"),
            AuditAction::TotpRecoveryCodeUsed => write!(f, "totp_recovery_code_used"),
            AuditAction::TotpRecoveryCodesSeen => write!(f, "totp_recovery_codes_seen"),
            AuditAction::SudoEnabled => write!(f, "sudo_enabled"),
            AuditAction::OauthApplicationCreated => write!(f, "oauth_application_created"),
            AuditAction::OauthApplicationUpdated => write!(f, "oauth_application_updated"),
            AuditAction::OauthApplicationDeleted => write!(f, "oauth_application_deleted"),
            AuditAction::OauthApplicationKeysRotated => write!(f, "oauth_application_keys_rotated"),
            AuditAction::OauthAuthorizationIntiated => write!(f, "oauth_authorization_initiated"),
            AuditAction::OauthAuthorizationApproved => write!(f, "oauth_authorization_approved"),
            AuditAction::OauthAuthorizationDenied => write!(f, "oauth_authorization_denied"),
            AuditAction::OauthAuthorizationRevoked => write!(f, "oauth_authorization_revoked"),
            AuditAction::OauthAuthorizationUpdated => write!(f, "oauth_authorization_updated"),
            AuditAction::OauthAuthorizationsRevoked => write!(f, "oauth_authorizations_revoked"),
            AuditAction::OauthTokenRevoked => write!(f, "oauth_token_revoked"),
            AuditAction::UserUpdated => write!(f, "user_updated"),
        }
    }
}

#[derive(Debug)]
pub enum ResourceType {
    User,
    OauthApplication,
    OauthAuthorization,
    Passkey,
    Session,
    Totp,
}

impl Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceType::User => write!(f, "user"),
            ResourceType::OauthApplication => write!(f, "oauth_application"),
            ResourceType::OauthAuthorization => write!(f, "oauth_authorization"),
            ResourceType::Passkey => write!(f, "passkey"),
            ResourceType::Session => write!(f, "session"),
            ResourceType::Totp => write!(f, "totp"),
        }
    }
}

/// Buildable audit log entry
#[derive(Debug, typed_builder::TypedBuilder)]
pub struct AuditEntry {
    /// Who performed this action?
    actor: AuditActor,
    /// Who was affected by this action?
    user_id: UserId,
    /// ... what was the action?
    action: AuditAction,
    /// What type of resource was affected by this action?
    #[builder(default = None)]
    pub resource_type: Option<ResourceType>,
    /// What specific resource was affected by this action?
    #[builder(default = None)]
    pub resource_id: Option<UlidId>,
    /// What specific changes were made to the resource by this action?
    #[builder(default = vec![])]
    pub changes: Vec<AuditChange>,
    /// What was the IP addr of the actor when this action was performed?
    pub actor_ip: IpAddr,
    /// What was the aproximate location of the actor when this action was performed?
    pub actor_location: String,
    /// What was the user agent of the actor when this action was performed?
    pub actor_user_agent: String,
}

impl AuditEntry {
    pub async fn save(self, tx: &mut PgTransaction<'_>) -> anyhow::Result<()> {
        let model = DbAuditLog::builder()
            .actor_id(self.actor.id())
            .actor_type(self.actor.to_string())
            .user_id(self.user_id)
            .action(self.action.to_string())
            .resource_id(self.resource_id)
            .resource_type(self.resource_type.map(|v| v.to_string()))
            .changes(serde_json::to_value(self.changes).unwrap_or(serde_json::Value::Null))
            .actor_ip(self.actor_ip.to_string())
            .actor_location(self.actor_location)
            .actor_user_agent(self.actor_user_agent)
            .build();

        model.insert(tx).await?;

        Ok(())
    }
}
