use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{oauth_application::OauthApplicationId, user::UserId},
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

    // pub async fn upsert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
    //     sqlx::query!(
    //         "insert into
    //             oauth_authorizations (id, user_id, client_id, scopes, last_used_at, created_at, updated_at)
    //         values
    //             ($1, $2, $3, $4, $5, now(), now())
    //         on conflict (user_id, client_id) do update set
    //             scopes = excluded.scopes,
    //             last_used_at = excluded.last_used_at,
    //             updated_at = now()
    //         ",
    //         self.id as OauthAuthorizationId,
    //         self.user_id as UserId,
    //         self.client_id as OauthApplicationId,
    //         self.scopes,
    //         self.last_used_at,
    //     )
    //     .execute(&mut **transaction)
    //     .await?;

    //     Ok(())
    // }

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
            user_id as OauthAuthorizationId,
            client_id as OauthApplicationId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
