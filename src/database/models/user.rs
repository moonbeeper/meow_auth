use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId};

pub type UserId = UlidId;
pub type PIDUserId = UlidId;

// TODO: add pid to user for external clients or providers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct User {
    #[builder(default = UserId::new())]
    pub id: UserId,
    #[builder(default = PIDUserId::new())]
    pub pid: PIDUserId,
    pub login: String,
    pub email: String,
    #[builder(default = false)]
    pub email_verified: bool,
    #[builder(default = false)]
    pub totp_enabled: bool,
    #[builder(default = false)]
    pub has_webauthn: bool,
    #[builder(default = chrono::Utc::now())]
    pub login_updated_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                users (id, pid, login, email, login_updated_at ,created_at, updated_at)
             values
                ($1, $2, lower($3), lower($4), now(), now(), now())",
            self.id as UserId,
            self.pid as PIDUserId,
            self.login,
            self.email,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update users set
                login = $2,
                email = $3,
                email_verified = $4,
                totp_enabled = $5,
                has_webauthn = $6,
                login_updated_at = $7,
                updated_at = now()
             where id = $1",
            self.id as UserId,
            self.login,
            self.email,
            self.email_verified,
            self.totp_enabled,
            self.has_webauthn,
            self.login_updated_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(id: UserId, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                login,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                login_updated_at,
                created_at,
                updated_at
             from users where id = $1",
            id as UserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_pid(pid: PIDUserId, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                login,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                login_updated_at,
                created_at,
                updated_at
             from users where pid = $1",
            pid as PIDUserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_email(
        email: String,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                login,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                login_updated_at,
                created_at,
                updated_at
             from users where email = lower($1)",
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_login(
        login: String,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                login,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                login_updated_at,
                created_at,
                updated_at
             from users where login = lower($1)",
            login
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<UserId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                login,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                login_updated_at,
                created_at,
                updated_at
             from users where id = any($1)",
            &ids as &[UserId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }
}
