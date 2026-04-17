use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{user::UserId, user_session::UserSessionId},
};

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
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_auth_challenges_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthChallengeKind {
    #[default]
    Otp,
    Totp,
    Webauthn,
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
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_auth_challenges_state")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthChallengeState {
    #[default]
    Pending,
    Completed,
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
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "user_auth_challenges_purpose")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthChallengePurpose {
    #[default]
    Login,
    Sudo,
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
            self.kind as AuthChallengeKind,
            self.secret.as_ref(),
            self.state as AuthChallengeState,
            self.purpose as AuthChallengePurpose,
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
            self.state as AuthChallengeState,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: UserAuthChallengesId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                user_session_id as "session_id?: UserSessionId",
                kind as "kind: AuthChallengeKind",
                state as "state: AuthChallengeState",
                purpose as "purpose: AuthChallengePurpose",
                secret,
                expires_at,
                created_at,
                updated_at
            from user_auth_challenges where id = $1 and expires_at > now() and state = 'pending'::user_auth_challenges_state"#,
            id as UserAuthChallengesId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<UserAuthChallengesId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                user_session_id as "session_id?: UserSessionId",
                kind as "kind: AuthChallengeKind",
                state as "state: AuthChallengeState",
                purpose as "purpose: AuthChallengePurpose",
                secret,
                expires_at,
                created_at,
                updated_at
            from user_auth_challenges where id = any($1) and expires_at > now() and state = 'pending'::user_auth_challenges_state"#,
            &ids as &[UserAuthChallengesId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_id_and_session_id(
        id: UserAuthChallengesId,
        session_id: UserSessionId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                user_session_id as "session_id?: UserSessionId",
                kind as "kind: AuthChallengeKind",
                state as "state: AuthChallengeState",
                purpose as "purpose: AuthChallengePurpose",
                secret,
                expires_at,
                created_at,
                updated_at
                from user_auth_challenges where id = $1 and user_session_id = $2 and expires_at > now() and state = 'pending'::user_auth_challenges_state"#,
            id as UserAuthChallengesId,
            session_id as UserSessionId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
