use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

pub type OauthApplicationId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthApplication {
    #[builder(default = OauthApplicationId::new())]
    pub id: OauthApplicationId,
    pub user_id: UserId,
    pub name: String,
    pub redirect_uri: String,
    pub secret: Vec<u8>,
    #[builder(default = false)]
    pub public: bool,
    #[builder(default = 0)]
    pub scopes: i64,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl OauthApplication {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                oauth_applications (id, user_id, name, redirect_uri, secret, public, scopes, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, $6, $7, now(), now())",
            self.id as OauthApplicationId,
            self.user_id as UserId,
            self.name,
            self.redirect_uri,
            self.secret,
            self.public,
            self.scopes,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update oauth_applications set
                name = $2,
                redirect_uri = $3,
                secret = $4,
                public = $5,
                scopes = $6,
                updated_at = now()
             where id = $1",
            self.id as OauthApplicationId,
            self.name,
            self.redirect_uri,
            self.secret,
            self.public,
            self.scopes,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: OauthApplicationId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                name,
                redirect_uri,
                secret,
                public,
                scopes,
                created_at,
                updated_at
             from oauth_applications where id = $1",
            id as OauthApplicationId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_user_id(
        user_id: UserId,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                redirect_uri,
                name,
                secret,
                public,
                scopes,
                created_at,
                updated_at
             from oauth_applications where id = $1",
            user_id as UserId
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }
}
