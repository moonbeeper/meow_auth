use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{oauth_application::OauthApplicationId, user::UserId},
};

// what the fuck, HWO DID I SET THIS TO 'String' what. stupid goofy ass bird brain I AM
pub type OauthTokenId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthToken {
    #[builder(default = OauthTokenId::new())]
    pub id: OauthTokenId,
    pub user_id: UserId,
    pub client_id: OauthApplicationId,
    pub token: String,
    #[builder(default = 0)]
    pub scopes: i64,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl OauthToken {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                oauth_tokens (id, user_id, client_id, token, scopes, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, now(), now())",
            self.id as OauthTokenId,
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.token,
            self.scopes,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_all_by_user_and_client_id(
        user_id: UserId,
        client_id: OauthApplicationId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_tokens where user_id = $1 and client_id = $2",
            user_id as UserId,
            client_id as OauthApplicationId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_token(id: String, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                client_id,
                token,
                scopes,
                created_at,
                updated_at
             from oauth_tokens where token = $1",
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }
}
