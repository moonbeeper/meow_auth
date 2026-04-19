use serde_json::json;
use sqlx::PgPool;

use crate::mailer::{
    error::MailerResult,
    resources::{EmailTemplate, MailerTemplate},
};

pub enum EmailVerificationCodeKind {
    Login,
    Register,
    Verification,
}

impl EmailVerificationCodeKind {
    fn to_str<'a>(&self) -> &'a str {
        match self {
            EmailVerificationCodeKind::Login => "login",
            EmailVerificationCodeKind::Register => "register",
            EmailVerificationCodeKind::Verification => "verification",
        }
    }
}

pub struct AuthMailer;

impl MailerTemplate for AuthMailer {}

impl AuthMailer {
    pub async fn new_session(login: String, to: String, db: &PgPool) -> MailerResult<()> {
        Self::mail_template(
            "New session".to_string(),
            to,
            EmailTemplate {
                base: "new_session",
                data: json!({
                  "user": {
                      "login": login,
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn verification_code(
        code: String,
        kind: EmailVerificationCodeKind,
        login: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        let template = match kind {
            EmailVerificationCodeKind::Login => "login_verification",
            EmailVerificationCodeKind::Register | EmailVerificationCodeKind::Verification => {
                "verification_code"
            }
        };

        Self::mail_template(
            format!("{code} is your verification code"),
            to,
            EmailTemplate {
                base: template,
                data: json!({
                  "user": {
                      "login": login,
                  },
                  "code": code,
                  "kind": kind.to_str(),
                }),
            },
            db,
        )
        .await
    }

    pub async fn totp_enabled(login: String, to: String, db: &PgPool) -> MailerResult<()> {
        Self::mail_template(
            "2FA has been enabled".to_string(),
            to,
            EmailTemplate {
                base: "totp_enabled",
                data: json!({
                  "user": {
                      "login": login,
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn totp_disabled(login: String, to: String, db: &PgPool) -> MailerResult<()> {
        Self::mail_template(
            "2FA has been disabled".to_string(),
            to,
            EmailTemplate {
                base: "totp_disabled",
                data: json!({
                  "user": {
                      "login": login,
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn totp_recovery_codes_seen(
        login: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "2FA recovery codes have been seen".to_string(),
            to,
            EmailTemplate {
                base: "totp_recovery_codes_seen",
                data: json!({
                  "user": {
                      "login": login,
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn totp_recovery_code_used(
        login: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "2FA recovery code used".to_string(),
            to,
            EmailTemplate {
                base: "totp_recovery_code_used",
                data: json!({
                  "user": {
                      "login": login,
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn webauthn_registered(
        login: String,
        passkey_name: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "A Passkey has been added to your account".to_string(),
            to,
            EmailTemplate {
                base: "webauthn_registered",
                data: json!({
                  "user": {
                      "login": login,
                  },
                  "passkey": {
                      "name": passkey_name
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn webauthn_removed(
        login: String,
        passkey_name: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "A Passkey has been removed from your account".to_string(),
            to,
            EmailTemplate {
                base: "webauthn_removed",
                data: json!({
                  "user": {
                      "login": login,
                  },
                  "passkey": {
                      "name": passkey_name
                  }
                }),
            },
            db,
        )
        .await
    }

    pub async fn webauthn_compromised(
        login: String,
        passkey_name: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "One of your Passkeys have been compromised".to_string(),
            to,
            EmailTemplate {
                base: "webauthn_compromised",
                data: json!({
                  "user": {
                      "login": login,
                  },
                  "passkey": {
                      "name": passkey_name
                  }
                }),
            },
            db,
        )
        .await
    }
}
