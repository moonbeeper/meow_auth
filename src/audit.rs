use std::fmt::Display;

use sqlx::PgTransaction;

use crate::database::models::{audit_log::AuditLog, user::UserId};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum AuditAction {
    SessionCreated, // should add metadata for what was used to create it (webauthn or otp)
    SessionDeleted,
    SessionsDeleted, // close all sessions
    EmailChangeRequested,
    EmailChanged,
    LoginChanged,
    AccountCreated,
    AccountDeleted, // TODO: THE HANDLER AAAGH GOD please dont incinerate me
    TotpEnabled,
    PasskeyAdded,
    PasskeyRemoved,
    PasskeyRenamed, // TODO: that's another one to do. easy tho
    PasskeyDisabled,
    TotpDisabled,
    TotpRecoveryCodesUsed,
    TotpRecoveryCodesSeen,
    SudoEnabled, // should add metadata for what was used to enable it (like the session)
}

impl Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::SessionCreated => write!(f, "session_created"),
            AuditAction::SessionDeleted => write!(f, "session_deleted"),
            AuditAction::SessionsDeleted => write!(f, "sessions_deleted"),
            AuditAction::EmailChangeRequested => write!(f, "email_change_requested"),
            AuditAction::EmailChanged => write!(f, "email_changed"),
            AuditAction::LoginChanged => write!(f, "login_changed"),
            AuditAction::AccountCreated => write!(f, "account_created"),
            AuditAction::AccountDeleted => write!(f, "account_deleted"),
            AuditAction::TotpEnabled => write!(f, "totp_enabled"),
            AuditAction::PasskeyAdded => write!(f, "passkey_added"),
            AuditAction::PasskeyRemoved => write!(f, "passkey_removed"),
            AuditAction::PasskeyRenamed => write!(f, "passkey_renamed"),
            AuditAction::PasskeyDisabled => write!(f, "passkey_disabled"),
            AuditAction::TotpDisabled => write!(f, "totp_disabled"),
            AuditAction::TotpRecoveryCodesUsed => write!(f, "totp_recovery_codes_used"),
            AuditAction::TotpRecoveryCodesSeen => write!(f, "totp_recovery_codes_seen"),
            AuditAction::SudoEnabled => write!(f, "sudo_enabled"),
        }
    }
}

pub async fn log(
    user_id: UserId,
    action: AuditAction,
    metadata: Option<serde_json::Value>,
    tx: &mut PgTransaction<'_>,
) -> anyhow::Result<()> {
    let metadata = metadata.unwrap_or(serde_json::json!({}));

    let model = AuditLog::builder()
        .user_id(user_id)
        .action(action.to_string())
        .metadata(metadata)
        .build();

    model.insert(tx).await?;
    Ok(())
}
