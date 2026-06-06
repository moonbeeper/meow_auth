use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

pub type UserWebauthnId = UlidId;
pub type PIDUserWebauthnId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserWebauthn {
    #[builder(default = UserWebauthnId::new())]
    pub id: UserWebauthnId,
    pub user_id: UserId,
    #[builder(default = PIDUserWebauthnId::new())]
    pub pid: PIDUserWebauthnId,
    #[builder(default = true)]
    pub enabled: bool,
    #[builder(default = "My Passkey".to_string())]
    pub display_name: String,
    pub credential_id: Vec<u8>,
    #[builder(default = Some(uuid::Uuid::from_u128(0)))]
    pub aaguid: Option<uuid::Uuid>,
    #[builder(default = 1)]
    pub counter: i32,
    pub big_data: serde_json::Value,
    #[builder(default = None)]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    #[builder(default = None)]
    pub disabled_at: Option<chrono::DateTime<chrono::Utc>>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserWebauthn {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                user_webauthn (id, user_id, pid, display_name, credential_id, aaguid, big_data, last_used_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, $6, $7, $8, now(), now())",
            self.id as UserWebauthnId,
            self.user_id as UserId,
            self.pid as PIDUserWebauthnId,
            self.display_name,
            self.credential_id,
            self.aaguid,
            self.big_data,
            self.last_used_at.as_ref()
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update user_webauthn set
                big_data = $2,
                counter = $3,
                enabled = $4,
                last_used_at = now(),
                updated_at = now(),
                disabled_at = case when enabled then null else now() end
             where id = $1",
            self.id as UserWebauthnId,
            self.big_data,
            self.counter,
            self.enabled
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_webauthn where id = $1",
            self.id as UserWebauthnId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_many_by_id(
        ids: Vec<UserWebauthnId>,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_webauthn where id = any($1)",
            &ids as &[UserWebauthnId]
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_by_pid(
        id: UserWebauthnId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_webauthn where pid = $1",
            id as PIDUserWebauthnId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: UserWebauthnId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where id = $1",
            id as UserWebauthnId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_pid(
        id: PIDUserWebauthnId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where pid = $1",
            id as PIDUserWebauthnId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_ids(
        ids: Vec<UserWebauthnId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where id = any($1)",
            &ids as &[UserWebauthnId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_user_id(
        id: UserId,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where user_id = $1",
            id as UserId
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_user_id(id: UserId, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where user_id = $1",
            id as UserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_credential_id(
        id: &[u8],
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                pid,
                enabled,
                display_name,
                credential_id,
                aaguid,
                counter,
                big_data,
                last_used_at,
                disabled_at,
                created_at,
                updated_at
             from user_webauthn where credential_id = $1",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn get_count_by_user_id(id: UserId, pool: &PgPool) -> Result<usize, DatabaseError> {
        let data = sqlx::query!(
            "select count(*) from user_webauthn where user_id = $1",
            id as UserId
        )
        .fetch_one(pool)
        .await?;

        Ok(data.count.unwrap_or_default() as usize)
    }
}
