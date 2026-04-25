use sqlx::PgTransaction;
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId};

pub type UserSignupId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserSignup {
    #[builder(default = UserSignupId::new())]
    pub id: UserSignupId,
    pub login: String,
    pub email: String,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(15))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl UserSignup {
    pub async fn upsert(&self, transaction: &mut PgTransaction<'_>) -> Result<Self, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "insert into
                user_signups (id, login, email, created_at, expires_at)
            values
                ($1, lower($2), lower($3), now(), $4)
            on conflict (login) do update set
                login = excluded.login,
                email = excluded.email,
                created_at = now(),
                expires_at = $4
            returning
                id,
                login,
                email,
                created_at,
                expires_at
            ",
            self.id as UserSignupId,
            self.login,
            self.email,
            self.expires_at
        )
        .fetch_one(&mut **transaction)
        .await?;

        Ok(data)
    }

    pub async fn delete_all_by_email_and_login(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_signups where (email = lower($1) or login = lower($2)) and expires_at > now()",
            self.email,
            self.login,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn take_by_id(
        id: UserSignupId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "delete from user_signups
                where id = $1 and expires_at > now()
                returning
                    id,
                    login,
                    email,
                    created_at,
                    expires_at
            ",
            id as UserSignupId
        )
        .fetch_optional(&mut **transaction)
        .await?;

        Ok(data)
    }
}
