use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{id::UlidId, models::user::UserId};

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum LoginFlowKind {
    #[default]
    Unknown,
    Otp,
}

impl LoginFlowKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginFlowKind::Otp => "otp",
            LoginFlowKind::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "otp" => LoginFlowKind::Otp,
            _ => Self::default(),
        }
    }
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum LoginFlowState {
    #[default]
    Pending,
    Completed,
    Expired,
}

impl LoginFlowState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginFlowState::Pending => "pending",
            LoginFlowState::Completed => "completed",
            LoginFlowState::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => LoginFlowState::Pending,
            "completed" => LoginFlowState::Completed,
            "expired" => LoginFlowState::Expired,
            _ => Self::default(),
        }
    }
}

pub type UserLoginRequestId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserLoginRequest {
    #[builder(default = UserLoginRequestId::new())]
    pub id: UserLoginRequestId,
    pub user_id: UserId,
    pub kind: LoginFlowKind,
    pub secret: Option<String>,
    #[builder(default = LoginFlowState::default())]
    pub state: LoginFlowState,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserLoginRequest {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "insert into
                user_login_requests (id, user_id, kind, secret, state, expires_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, $6, now(), now())",
            self.id as UserLoginRequestId,
            self.user_id as UserId,
            self.kind.as_str(),
            self.secret.as_ref(),
            self.state.as_str(),
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "update user_login_requests set state = $2 where id = $1",
            self.id as UserLoginRequestId,
            self.state.as_str()
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: UserLoginRequestId,
        pool: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let data = sqlx::query!(
            "select * from user_login_requests where id = $1 and expires_at > now() and state = 'pending'",
            id as UserLoginRequestId
        )
        .fetch_optional(pool)
        .await?
        .map(|v| Self {
            id,
            user_id: v.user_id.into(),
            kind: LoginFlowKind::from_str(&v.kind),
            secret: v.secret,
            state: LoginFlowState::from_str(&v.state),
            expires_at: v.expires_at,
            created_at: v.created_at,
            updated_at: v.updated_at,
        });

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<UserLoginRequestId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let data = sqlx::query!(
            "select * from user_login_requests where id = ANY($1) and expires_at > now() and state = 'pending'",
            &ids as &[UserLoginRequestId]
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|v| Self {
            id: v.id.into(),
            user_id: v.user_id.into(),
            kind: LoginFlowKind::from_str(&v.kind),
            secret: v.secret,
            state: LoginFlowState::from_str(&v.state),
            expires_at: v.expires_at,
            created_at: v.created_at,
            updated_at: v.updated_at,
        })
        .collect();

        Ok(data)
    }
}
