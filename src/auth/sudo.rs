use sqlx::{PgPool, PgTransaction};

use crate::{
    database::models::{
        user::{User, UserId},
        user_auth_challenge::{AuthChallengeKind, AuthChallengePurpose, UserAuthChallenges},
        user_session::{UserSession, UserSessionId},
    },
    http::middleware::auth_manager::AuthContext,
    settings::Settings,
};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SudoOption {
    #[default]
    Otp,
    Totp,
    Passkey,
}

impl From<SudoOption> for AuthChallengeKind {
    fn from(value: SudoOption) -> Self {
        match value {
            SudoOption::Otp => AuthChallengeKind::Otp,
            SudoOption::Totp => AuthChallengeKind::Totp,
            SudoOption::Passkey => AuthChallengeKind::Otp, // for passkeys
        }
    }
}

impl From<AuthChallengeKind> for SudoOption {
    fn from(value: AuthChallengeKind) -> Self {
        match value {
            AuthChallengeKind::Otp => SudoOption::Otp,
            AuthChallengeKind::Totp => SudoOption::Totp,
            AuthChallengeKind::Webauthn => SudoOption::Passkey,
        }
    }
}

// TODO: hey i am double fetching the user :(
pub async fn get_available_options(user_id: UserId, db: &PgPool) -> Vec<SudoOption> {
    let mut options = Vec::new();
    if let Ok(Some(user)) = User::find_by_id(user_id, db).await {
        if user.totp_enabled {
            options.push(SudoOption::Totp);
        } else {
            options.push(SudoOption::Otp);
        }

        if user.has_webauthn {
            options.push(SudoOption::Passkey);
        }
    }

    options
}

pub async fn has_sudo_option(kind: SudoOption, user_id: UserId, db: &PgPool) -> bool {
    let sudo_options = get_available_options(user_id, db).await;
    sudo_options.contains(&kind)
}

pub async fn enable_sudo_tx(
    auth: &AuthContext,
    tx: &mut PgTransaction<'_>,
    settings: &Settings,
) -> anyhow::Result<()> {
    let Some(mut session) = UserSession::find_by_id(auth.session_id(), &mut **tx).await? else {
        anyhow::bail!("session wasnt found wtf")
    };

    session.sudo_expires_at = Some(
        chrono::Utc::now() + chrono::Duration::seconds(settings.session.sudo_expire_age_seconds),
    );
    session.update(tx).await?;

    Ok(())
}

pub fn is_flow_correct(
    flow: &UserAuthChallenges,
    kind: AuthChallengeKind,
    session_id: UserSessionId,
) -> bool {
    if !super::is_flow_correct(flow, Some(kind), Some(AuthChallengePurpose::Sudo)) {
        return false;
    }

    if flow.session_id != Some(session_id) {
        return false;
    }

    if flow.user_id.is_none() {
        return false;
    }

    true
}
