use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

pub type UserTotpId = UlidId;

// TODO: add pid to user for external clients or providers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserTotp {
    #[builder(default = UserTotpId::new())]
    pub id: UserTotpId,
    pub user_id: UserId,
    pub recovery_secret: Vec<u8>,
    pub recovery_secret_nonce: Vec<u8>,
    #[builder(default = 0)]
    pub recovery_used: i32,
    pub secret: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    #[builder(default = None)]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserTotp {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                user_totp (
                id,
                user_id,
                recovery_secret,
                recovery_secret_nonce,
                recovery_used,
                secret,
                secret_nonce,
                last_used_at,
                created_at,
                updated_at
                )
             values
                ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())",
            self.id as UserTotpId,
            self.user_id as UserId,
            self.recovery_secret,
            self.recovery_secret_nonce,
            self.recovery_used,
            self.secret,
            self.secret_nonce,
            self.last_used_at.as_ref()
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update user_totp set
                recovery_used = $2,
                last_used_at = now(),
                updated_at = now()
             where id = $1",
            self.id as UserTotpId,
            self.recovery_used
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!("delete from user_totp where id = $1", self.id as UserTotpId)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn delete_all_by_user(
        id: UserId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!("delete from user_totp where user_id = $1", id as UserId)
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn find_by_id(id: UserTotpId, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                recovery_secret,
                recovery_secret_nonce,
                recovery_used,
                secret,
                secret_nonce,
                last_used_at,
                created_at,
                updated_at
             from user_totp where id = $1",
            id as UserTotpId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_one_by_user(
        id: UserId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                recovery_secret,
                recovery_secret_nonce,
                recovery_used,
                secret,
                secret_nonce,
                last_used_at,
                created_at,
                updated_at
             from user_totp where user_id = $1 limit 1",
            id as UserId
        )
        .fetch_one(pool)
        .await;

        match data {
            Ok(data) => Ok(Some(data)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e)?,
        }
    }

    pub async fn find_many_by_id(
        ids: Vec<UserTotpId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                recovery_secret,
                recovery_secret_nonce,
                recovery_used,
                secret,
                secret_nonce,
                last_used_at,
                created_at,
                updated_at
             from user_totp where id = any($1)",
            &ids as &[UserTotpId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub const fn is_recovery_code_used(&self, n: usize) -> bool {
        ((1 << n) & self.recovery_used) != 0
    }

    pub const fn mark_recovery_code_used(&mut self, n: usize) {
        self.recovery_used |= 1 << n;
    }

    pub const fn remaining_recovery_codes(&self) -> u32 {
        16 - self.recovery_used.count_ones()
    }
}
