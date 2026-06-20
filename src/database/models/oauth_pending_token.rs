use sqlx::PgTransaction;
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    models::{oauth_application::OauthApplicationId, user::UserId},
};

pub type OauthPendingTokenId = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthPendingToken {
    #[builder(default = nanoid::nanoid!(32))]
    pub code: OauthPendingTokenId,
    pub user_id: UserId,
    pub client_id: OauthApplicationId,
    #[builder(default = 0)]
    pub scopes: i64,
    pub code_challenge: String,
    pub state: Option<String>,
    pub nonce: Option<String>, // openid id token
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(15))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OauthPendingToken {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into oauth_pending_tokens (
                code,
                user_id,
                client_id,
                scopes,
                code_challenge,
                state,
                nonce,
                expires_at
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            ",
            self.code,
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.scopes,
            self.code_challenge,
            self.state,
            self.nonce,
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_all(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_pending_tokens where user_id = $1 and client_id = $2",
            self.user_id as UserId,
            self.client_id as OauthApplicationId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn take_by_id(
        id: OauthPendingTokenId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "delete from oauth_pending_tokens
                where code = $1 and expires_at > now()
                returning
                    code,
                    user_id,
                    client_id,
                    scopes,
                    code_challenge,
                    state,
                    nonce,
                    expires_at,
                    created_at
            ",
            id
        )
        .fetch_optional(&mut **transaction)
        .await?;

        Ok(data)
    }
}
