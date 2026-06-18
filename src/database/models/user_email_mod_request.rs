use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

pub type UserEmailModificationRequestId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserEmailModificationRequest {
    #[builder(default = UserEmailModificationRequestId::new())]
    pub id: UserEmailModificationRequestId,
    pub user_id: UserId,
    pub current_email: String,
    #[builder(default = false)]
    pub current_email_verified: bool,
    pub current_email_token: Vec<u8>,
    pub new_email: String,
    #[builder(default = false)]
    pub new_email_verified: bool,
    pub new_email_token: Vec<u8>,
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(15))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserEmailModificationRequest {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                user_email_mod_requests (
                id,
                user_id,
                current_email,
                current_email_verified,
                current_email_token,
                new_email,
                new_email_verified,
                new_email_token,
                expires_at,
                created_at,
                updated_at
                )
             values
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), now())",
            self.id as UserEmailModificationRequestId,
            self.user_id as UserId,
            self.current_email,
            self.current_email_verified,
            self.current_email_token,
            self.new_email,
            self.new_email_verified,
            self.new_email_token,
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update user_email_mod_requests set
                current_email_verified = $2,
                new_email_verified = $3,
                updated_at = now()
             where id = $1",
            self.id as UserEmailModificationRequestId,
            self.current_email_verified,
            self.new_email_verified
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_email_mod_requests where id = $1",
            self.id as UserEmailModificationRequestId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_all_by_user(
        id: UserId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_email_mod_requests where user_id = $1",
            id as UserId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_token_and_user_id(
        token: &[u8],
        user_id: UserId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                current_email,
                current_email_verified,
                current_email_token,
                new_email,
                new_email_verified,
                new_email_token,
                expires_at,
                created_at,
                updated_at
             from user_email_mod_requests where user_id = $2 and (current_email_token = $1 or new_email_token = $1) and expires_at > now()",
            token,
            user_id as UserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
