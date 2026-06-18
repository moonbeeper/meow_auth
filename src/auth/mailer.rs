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

// TODO: awful, maybe merge those two?
pub enum NewEmailVerificationCodeKind {
    Current,
    New,
}

impl NewEmailVerificationCodeKind {
    fn to_str<'a>(&self) -> &'a str {
        match self {
            Self::Current => "current",
            Self::New => "new",
        }
    }
}

pub struct AuthMailer;

impl MailerTemplate for AuthMailer {}

impl AuthMailer {
    pub async fn new_session(login: String, to: String, db: &PgPool) -> MailerResult<()> {
        Self::mail_template(
            "A new session has been created".to_string(),
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

    pub async fn new_account(login: String, to: String, db: &PgPool) -> MailerResult<()> {
        Self::mail_template(
            "Your account has been created".to_string(),
            to,
            EmailTemplate {
                base: "new_account",
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

        let subject = match kind {
            EmailVerificationCodeKind::Login => "sign in",
            EmailVerificationCodeKind::Register => "registration",
            EmailVerificationCodeKind::Verification => "verification",
        };

        Self::mail_template(
            format!("{code} is your {} code", subject),
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
            "Two factor authentication enabled".to_string(),
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
            "Two factor authentication disabled".to_string(),
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
            "Your Two factor authentication recovery codes were viewed".to_string(),
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
            "One of your Two factor authentication recovery codes was used".to_string(),
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
            "A Passkey was added to your account".to_string(),
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
            "A Passkey was removed from your account".to_string(),
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
            "One of your Passkeys may be compromised".to_string(),
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

    // TODO: should insert token in the frontend url to be able to use the button.
    pub async fn email_verification(
        kind: NewEmailVerificationCodeKind,
        token: String,
        new_address: String,
        login: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        let subject = match kind {
            NewEmailVerificationCodeKind::Current => "Verify your current email address",
            NewEmailVerificationCodeKind::New => "Verify your new email address",
        };

        Self::mail_template(
            subject.to_string(),
            to,
            EmailTemplate {
                base: "new_email_verification",
                data: json!({
                  "user": {
                      "login": login,
                      "new_address": new_address,
                      "token": token,
                  },
                  "kind": kind.to_str(),
                }),
            },
            db,
        )
        .await
    }

    pub async fn email_updated(
        kind: NewEmailVerificationCodeKind,
        new_address: String,
        login: String,
        to: String,
        db: &PgPool,
    ) -> MailerResult<()> {
        Self::mail_template(
            "Your account's email was updated".to_string(),
            to,
            EmailTemplate {
                base: "email_updated",
                data: json!({
                  "user": {
                      "login": login,
                      "new_address": new_address,
                  },
                  "kind": kind.to_str(),
                }),
            },
            db,
        )
        .await
    }
}
