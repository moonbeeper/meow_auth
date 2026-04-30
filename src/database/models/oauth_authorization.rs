use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{oauth_application::OauthApplicationId, user::UserId},
};

pub type OauthTokenId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthAuthorization {
    #[builder(default = OauthTokenId::new())]
    pub id: OauthTokenId,
    pub user_id: UserId,
    pub client_id: OauthApplicationId,
    #[builder(default = 0)]
    pub scopes: i64,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl OauthAuthorization {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                oauth_authorizations (id, user_id, client_id, scopes, created_at, updated_at)
             values
                ($1, $2, $3, $4, now(), now())",
            self.id as OauthTokenId,
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.scopes,
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
                created_at,
                updated_at
             from oauth_authorizations where user_id = $1 and client_id = $2",
            user_id as OauthTokenId,
            client_id as OauthApplicationId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
