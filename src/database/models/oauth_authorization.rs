use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{oauth_application::OauthApplicationId, user::UserId},
    pagination::{PaginatedId, PaginationResult},
};

pub type OauthAuthorizationId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthAuthorization {
    #[builder(default = OauthAuthorizationId::new())]
    pub id: OauthAuthorizationId,
    pub user_id: UserId,
    pub client_id: OauthApplicationId,
    #[builder(default = 0)]
    pub scopes: i64,
    #[builder(default = None)]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl OauthAuthorization {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                oauth_authorizations (id, user_id, client_id, scopes, last_used_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, now(), now())",
            self.id as OauthAuthorizationId,
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.scopes,
            self.last_used_at,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update oauth_authorizations set
                scopes = $2,
                last_used_at = now(),
                updated_at = now()
             where id = $1",
            self.id as OauthAuthorizationId,
            self.scopes,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_authorizations where id = $1",
            self.id as OauthAuthorizationId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_all_by_client_id(
        client_id: OauthApplicationId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_authorizations where client_id = $1",
            client_id as OauthApplicationId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_by_id(
        id: OauthAuthorizationId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_authorizations where id = $1",
            id as OauthAuthorizationId,
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
                user_id,
                client_id,
                scopes,
                last_used_at,
                created_at,
                updated_at
             from oauth_authorizations where id = $1",
            id as OauthAuthorizationId,
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_user_and_client_id(
        user_id: UserId,
        client_id: OauthApplicationId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                client_id,
                scopes,
                last_used_at,
                created_at,
                updated_at
             from oauth_authorizations where user_id = $1 and client_id = $2",
            user_id as UserId,
            client_id as OauthApplicationId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_user_id_paginated(
        user_id: UserId,
        from: Option<OauthApplicationId>,
        want_total: bool,
        pool: &PgPool,
    ) -> Result<PaginationResult<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                client_id,
                scopes,
                last_used_at,
                created_at,
                updated_at
             from oauth_authorizations where user_id = $1 and ($2::uuid is null or id::uuid > $2) order by created_at asc limit 20+1",
            user_id as UserId,
            from as Option<OauthAuthorizationId>
        )
        .fetch_all(pool)
        .await?;

        let total_rows = if want_total {
            Self::count_by_user_id(user_id, pool).await?
        } else {
            None
        };

        Ok(PaginationResult::new(data, total_rows))
    }

    pub async fn count_by_user_id(
        user_id: UserId,
        pool: &PgPool,
    ) -> Result<Option<i64>, DatabaseError> {
        let data = sqlx::query_scalar!(
            "select count(*) from oauth_authorizations where user_id = $1",
            user_id as UserId,
        )
        .fetch_one(pool)
        .await?;

        Ok(data)
    }
}

impl PaginatedId for OauthAuthorization {
    fn paginated_id(&self) -> UlidId {
        self.id
    }
}
