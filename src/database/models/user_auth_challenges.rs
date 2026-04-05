use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{user::UserId, user_session::UserSessionId},
};

// TODO: should swap to use a db enum and parse it with sqlx::from_row

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum AuthChallengeKind {
    #[default]
    Unknown,
    Otp,
    Totp,
}

impl AuthChallengeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthChallengeKind::Otp => "otp",
            AuthChallengeKind::Totp => "totp",
            AuthChallengeKind::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "otp" => AuthChallengeKind::Otp,
            "totp" => AuthChallengeKind::Totp,
            _ => Self::default(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum AuthChallengeState {
    #[default]
    Pending,
    Completed,
    Expired,
}

impl AuthChallengeState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "completed" => Self::Completed,
            "expired" => Self::Expired,
            _ => Self::default(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum AuthChallengePurpose {
    #[default]
    Login,
    Sudo,
}

impl AuthChallengePurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Sudo => "sudo",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "login" => Self::Login,
            "sudo" => Self::Sudo,
            _ => Self::default(),
        }
    }
}

pub type UserAuthChallengesId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserAuthChallenges {
    #[builder(default = UserAuthChallengesId::new())]
    pub id: UserAuthChallengesId,
    pub user_id: UserId,
    #[builder(default = None)]
    pub session_id: Option<UserSessionId>,
    pub kind: AuthChallengeKind,
    pub secret: Option<String>,
    #[builder(default = AuthChallengeState::default())]
    pub state: AuthChallengeState,
    #[builder(default = AuthChallengePurpose::default())]
    pub purpose: AuthChallengePurpose,
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(5))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserAuthChallenges {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                user_auth_challenges (id, user_id, user_session_id, kind, secret, state, purpose, expires_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())",
            self.id as UserAuthChallengesId,
            self.user_id as UserId,
            self.session_id as Option<UserSessionId>,
            self.kind.as_str(),
            self.secret.as_ref(),
            self.state.as_str(),
            self.purpose.as_str(),
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update user_auth_challenges set state = $2 where id = $1",
            self.id as UserAuthChallengesId,
            self.state.as_str()
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: UserAuthChallengesId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query!(
            "select * from user_auth_challenges where id = $1 and expires_at > now() and state = 'pending'",
            id as UserAuthChallengesId
        )
        .fetch_optional(pool)
        .await?
        .map(|v| Self {
            id,
            user_id: v.user_id.into(),
            session_id: v.user_session_id.as_ref().map(|v| v.into()),
            kind: AuthChallengeKind::from_str(&v.kind),
            secret: v.secret,
            state: AuthChallengeState::from_str(&v.state),
            purpose: AuthChallengePurpose::from_str(&v.purpose),
            expires_at: v.expires_at,
            created_at: v.created_at,
            updated_at: v.updated_at,
        });

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<UserAuthChallengesId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query!(
            "select * from user_auth_challenges where id = ANY($1) and expires_at > now() and state = 'pending'",
            &ids as &[UserAuthChallengesId]
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|v| Self {
            id: v.id.into(),
            user_id: v.user_id.into(),
            session_id: v.user_session_id.as_ref().map(|v| v.into()),
            kind: AuthChallengeKind::from_str(&v.kind),
            secret: v.secret,
            state: AuthChallengeState::from_str(&v.state),
            purpose: AuthChallengePurpose::from_str(&v.purpose),
            expires_at: v.expires_at,
            created_at: v.created_at,
            updated_at: v.updated_at,
        })
        .collect();

        Ok(data)
    }

    pub async fn find_by_id_and_session_id(
        id: UserAuthChallengesId,
        session_id: UserSessionId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query!(
            "select * from user_auth_challenges where id = $1 and user_session_id = $2 and expires_at > now() and state = 'pending'",
            id as UserAuthChallengesId,
            session_id as UserSessionId
        )
        .fetch_optional(pool)
        .await?
        .map(|v| Self {
            id,
            user_id: v.user_id.into(),
            session_id: v.user_session_id.as_ref().map(|v| v.into()),
            kind: AuthChallengeKind::from_str(&v.kind),
            secret: v.secret,
            state: AuthChallengeState::from_str(&v.state),
            purpose: AuthChallengePurpose::from_str(&v.purpose),
            expires_at: v.expires_at,
            created_at: v.created_at,
            updated_at: v.updated_at,
        });

        Ok(data)
    }
}
