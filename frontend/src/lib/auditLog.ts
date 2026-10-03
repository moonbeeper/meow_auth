export enum AuditAction {
    SessionCreated = "session_created",
    SessionRevoked = "session_deleted",
    /** When all sessions are revoked */
    SessionsRevoked = "sessions_deleted", // close all sessions
    EmailChangeRequested = "email_change_requested",
    EmailChanged = "email_changed",
    NameChanged = "login_changed",
    AccountCreated = "account_created",
    /** TODO: handler not mad in backend */
    AccountDeleted = "account_deleted",
    TotpEnabled = "totp_enabled",
    PasskeyAdded = "passkey_added",
    PasskeyRemoved = "passkey_removed",
    PasskeyRenamed = "passkey_renamed",
    PasskeyDisabled = "passkey_disabled",
    TotpDisabled = "totp_disabled",
    TotpRecoveryCodeUsed = "totp_recovery_code_used",
    TotpRecoveryCodesSeen = "totp_recovery_codes_seen",
    SudoEnabled = "sudo_enabled",
    OauthApplicationCreated = "oauth_application_created",
    OauthApplicationUpdated = "oauth_application_updated",
    OauthApplicationDeleted = "oauth_application_deleted",
    OauthApplicationKeysRotated = "oauth_application_keys_rotated",
    OauthAuthorizationIntiated = "oauth_authorization_initiated",
    OauthAuthorizationApproved = "oauth_authorization_approved",
    OauthAuthorizationDenied = "oauth_authorization_denied",
    OauthAuthorizationRevoked = "oauth_authorization_revoked",
    OauthAuthorizationUpdated = "oauth_authorization_updated",
    OauthAuthorizationsRevoked = "oauth_authorizations_revoked",
    UserUpdated = "user_updated"
}

export enum AuditActionCategory {
    OAuth = "OAuth",
    Account = "Account",
    Session = "Session",
    Security = "Security"
}

const humanReadableAuditActions: Record<AuditAction, string> = {
    [AuditAction.SessionCreated]: "Created session",
    [AuditAction.SessionRevoked]: "Deleted session",
    [AuditAction.SessionsRevoked]: "Deleted all sessions",
    [AuditAction.EmailChangeRequested]: "Requested email change",
    [AuditAction.EmailChanged]: "Changed email",
    [AuditAction.NameChanged]: "Changed name",
    [AuditAction.AccountCreated]: "Created account",
    [AuditAction.AccountDeleted]: "Deleted account",
    [AuditAction.TotpEnabled]: "Enabled two-factor auth",
    [AuditAction.PasskeyAdded]: "Added passkey",
    [AuditAction.PasskeyRemoved]: "Removed passkey",
    [AuditAction.PasskeyRenamed]: "Renamed passkey",
    [AuditAction.PasskeyDisabled]: "Disabled passkey",
    [AuditAction.TotpDisabled]: "Disabled two-factor auth",
    [AuditAction.TotpRecoveryCodeUsed]: "Used 2FA recovery code",
    [AuditAction.TotpRecoveryCodesSeen]: "Viewed 2FA recovery codes",
    [AuditAction.SudoEnabled]: "Enabled sudo mode",
    [AuditAction.OauthApplicationCreated]: "Created app",
    [AuditAction.OauthApplicationUpdated]: "Updated app",
    [AuditAction.OauthApplicationDeleted]: "Deleted app",
    [AuditAction.OauthApplicationKeysRotated]: "Rotated app keys",
    [AuditAction.OauthAuthorizationIntiated]: "Started OAuth authorization",
    [AuditAction.OauthAuthorizationApproved]: "Approved OAuth authorization",
    [AuditAction.OauthAuthorizationDenied]: "Denied OAuth authorization",
    [AuditAction.OauthAuthorizationRevoked]: "Revoked OAuth authorization",
    [AuditAction.OauthAuthorizationUpdated]: "Updated OAuth authorization",
    [AuditAction.OauthAuthorizationsRevoked]: "Revoked all OAuth authorizations",
    [AuditAction.UserUpdated]: "Updated user"
};

export const auditActionCategory: Record<AuditAction, AuditActionCategory> = {
    [AuditAction.SessionCreated]: AuditActionCategory.Session,
    [AuditAction.SessionRevoked]: AuditActionCategory.Session,
    [AuditAction.SessionsRevoked]: AuditActionCategory.Session,
    [AuditAction.EmailChangeRequested]: AuditActionCategory.Account,
    [AuditAction.EmailChanged]: AuditActionCategory.Account,
    [AuditAction.NameChanged]: AuditActionCategory.Account,
    [AuditAction.AccountCreated]: AuditActionCategory.Account,
    [AuditAction.AccountDeleted]: AuditActionCategory.Account,
    [AuditAction.TotpEnabled]: AuditActionCategory.Security,
    [AuditAction.PasskeyAdded]: AuditActionCategory.Security,
    [AuditAction.PasskeyRemoved]: AuditActionCategory.Security,
    [AuditAction.PasskeyRenamed]: AuditActionCategory.Security,
    [AuditAction.PasskeyDisabled]: AuditActionCategory.Security,
    [AuditAction.TotpDisabled]: AuditActionCategory.Security,
    [AuditAction.TotpRecoveryCodeUsed]: AuditActionCategory.Security,
    [AuditAction.TotpRecoveryCodesSeen]: AuditActionCategory.Security,
    [AuditAction.SudoEnabled]: AuditActionCategory.Account,
    [AuditAction.OauthApplicationCreated]: AuditActionCategory.OAuth,
    [AuditAction.OauthApplicationUpdated]: AuditActionCategory.OAuth,
    [AuditAction.OauthApplicationDeleted]: AuditActionCategory.OAuth,
    [AuditAction.OauthApplicationKeysRotated]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationIntiated]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationApproved]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationDenied]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationRevoked]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationUpdated]: AuditActionCategory.OAuth,
    [AuditAction.OauthAuthorizationsRevoked]: AuditActionCategory.OAuth,
    [AuditAction.UserUpdated]: AuditActionCategory.Account
};

export const intoAuditAction = (action: string): AuditAction => {
    return Object.values(AuditAction).find((a) => a === action) ?? AuditAction.UserUpdated;
};

export const actionLabel = (action: AuditAction): string => {
    return humanReadableAuditActions[action] ?? "Unknown action";
};

export const actionCategory = (action: AuditAction): string => {
    return auditActionCategory[action] ?? "Unknown";
};
