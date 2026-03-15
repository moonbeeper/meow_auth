use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{id::UlidId, models::user::UserId};

pub type UserSessionId = UlidId;
pub type PIDUserSessionId = UlidId;

// remember: PIDs will be shown to the user to close their sessions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserSession {
    #[builder(default = UserSessionId::new())]
    pub id: UserSessionId,
    pub user_id: UserId,
    #[builder(default = PIDUserSessionId::new())]
    pub pid: PIDUserSessionId,
    pub active_expires_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserSession {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "insert into
                user_sessions (id, user_id, pid, active_expires_at, expires_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, now(), now())",
            self.id as UserSessionId,
            self.user_id as UserId,
            self.pid as PIDUserSessionId,
            self.active_expires_at,
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "update user_sessions set
                active_expires_at = $2,
                updated_at = now()
             where id = $1",
            self.id as UserSessionId,
            self.active_expires_at,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: UserSessionId, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                active_expires_at,
                expires_at,
                created_at,
                updated_at
             from user_sessions where id = $1",
            id as UserSessionId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<UserSessionId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                active_expires_at,
                expires_at,
                created_at,
                updated_at
             from user_sessions where id = any($1)",
            &ids as &[UserSessionId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_user_id(id: UserId, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                active_expires_at,
                expires_at,
                created_at,
                updated_at
             from user_sessions where user_id = $1",
            id as UserSessionId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_pid(
        id: PIDUserSessionId,
        pool: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                active_expires_at,
                expires_at,
                created_at,
                updated_at
             from user_sessions where pid = $1",
            id as PIDUserSessionId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
