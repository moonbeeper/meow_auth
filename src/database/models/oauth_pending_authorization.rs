use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::{
        oauth_application::OauthApplicationId, oauth_authorization::OauthAuthorizationId,
        user::UserId, user_session::UserSessionId,
    },
};

pub type OauthPendingAuthorizationId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct OauthPendingAuthorization {
    #[builder(default = OauthPendingAuthorizationId::new())]
    pub id: OauthPendingAuthorizationId,
    pub user_id: UserId,
    pub client_id: OauthApplicationId,
    pub user_session: UserSessionId,
    #[builder(default = None)]
    pub old_authorization_id: Option<OauthAuthorizationId>,
    #[builder(default = None)]
    pub old_scopes: Option<i64>,
    #[builder(default = 0)]
    pub requested_scopes: i64,
    pub code_challenge: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub redirect_url: String,
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(15))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OauthPendingAuthorization {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into oauth_pending_authorizations (
                id,
                user_id,
                client_id,
                user_session,
                old_authorization_id,
                old_scopes,
                requested_scopes,
                code_challenge,
                state,
                nonce,
                redirect_url,
                expires_at
            )
            values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ",
            self.id as OauthPendingAuthorizationId,
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.user_session as UserSessionId,
            self.old_authorization_id as Option<OauthAuthorizationId>,
            self.old_scopes,
            self.requested_scopes,
            self.code_challenge,
            self.state,
            self.nonce,
            self.redirect_url,
            self.expires_at
        )
        .execute(&mut **transaction) // wtf, when did i put fetch_one here !?
        .await?;

        Ok(())
    }

    pub async fn delete_all(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from oauth_pending_authorizations where user_id = $1 and client_id = $2 and user_session = $3 and expires_at > now()",
            self.user_id as UserId,
            self.client_id as OauthApplicationId,
            self.user_session as UserSessionId
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: OauthPendingAuthorizationId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                client_id,
                user_session,
                old_authorization_id as "old_authorization_id?: OauthAuthorizationId",
                old_scopes,
                requested_scopes,
                code_challenge,
                state,
                nonce,
                redirect_url,
                expires_at,
                created_at
            from oauth_pending_authorizations where id = $1 and expires_at > now()"#,
            id as OauthPendingAuthorizationId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn take_by_id(
        id: OauthPendingAuthorizationId,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"delete from oauth_pending_authorizations
                where id = $1 and expires_at > now()
                returning
                    id,
                    user_id,
                    client_id,
                    user_session,
                    old_authorization_id as "old_authorization_id?: OauthAuthorizationId",
                    old_scopes,
                    requested_scopes,
                    code_challenge,
                    state,
                    nonce,
                    redirect_url,
                    expires_at,
                    created_at
            "#,
            id as OauthPendingAuthorizationId
        )
        .fetch_optional(&mut **transaction)
        .await?;

        Ok(data)
    }
}
