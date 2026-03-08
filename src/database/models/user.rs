use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::id::UlidId;

pub type UserId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct User {
    #[builder(default = UserId::new())]
    pub id: UserId,
    pub login: String,
    pub email: String,
    pub email_verified: bool,
    pub password_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
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

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
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

    pub async fn find_by_id(&self, id: UserId, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
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
        &self,
        ids: Vec<UserId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
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
}
