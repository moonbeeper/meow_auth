use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId};

pub type UserId = UlidId;

// TODO: add pid to user for external clients or providers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct User {
    #[builder(default = UserId::new())]
    pub id: UserId,
    pub login: String,
    pub email: String,
    #[builder(default = false)]
    pub email_verified: bool,
    #[builder(default = None)]
    pub password_hash: Option<String>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                users (id, login, email, created_at, updated_at)
             values
                ($1, lower($2), $3, now(), now())",
            self.id as UserId,
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
                password_hash = $5,
                updated_at = now()
             where id = $1",
            self.id as UserId,
            self.login,
            self.email,
            self.email_verified,
            self.password_hash.as_ref(),
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
                login,
                email,
                email_verified,
                password_hash,
                created_at,
                updated_at
             from users where id = $1",
            id as UserId
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
                login,
                email,
                email_verified,
                password_hash,
                created_at,
                updated_at
             from users where id = any($1)",
            &ids as &[UserId]
        )
        .fetch_all(pool)
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
                login,
                email,
                email_verified,
                password_hash,
                created_at,
                updated_at
             from users where email = $1",
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_email_and_login(
        email: String,
        login: String,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                login,
                email,
                email_verified,
                password_hash,
                created_at,
                updated_at
             from users where email = $1 or login = lower($2)",
            email,
            login
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
