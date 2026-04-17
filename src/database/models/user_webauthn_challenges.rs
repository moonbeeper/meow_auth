use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "webauthn_challenge_kind")]
#[sqlx(rename_all = "lowercase")]
pub enum WebauthnChallengeKind {
    #[default]
    Register,
    Authenticate,
}

pub type UserWebauthnChallengeId = UlidId; // holy that's long

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct UserWebauthnChallenge {
    #[builder(default = UserWebauthnChallengeId::new())]
    pub id: UserWebauthnChallengeId,
    pub user_id: UserId,
    pub kind: WebauthnChallengeKind,
    pub big_data: serde_json::Value,
    #[builder(default = chrono::Utc::now() + chrono::Duration::minutes(5))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl UserWebauthnChallenge {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                user_webauthn_challenges (id, user_id, kind, big_data, expires_at, created_at, updated_at)
             values
                ($1, $2, $3, $4, $5, now(), now())",
            self.id as UserWebauthnChallengeId,
            self.user_id as UserId,
            self.kind as WebauthnChallengeKind,
            self.big_data,
            self.expires_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_webauthn_challenges where id = $1",
            self.id as UserWebauthnChallengeId,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn delete_all_by_user(
        id: UserId,
        kind: WebauthnChallengeKind,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "delete from user_webauthn_challenges where user_id = $1 and kind = $2",
            id as UserId,
            kind as WebauthnChallengeKind
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_userid(
        id: UserId,
        kind: WebauthnChallengeKind,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                kind as "kind: WebauthnChallengeKind",
                big_data,
                expires_at,
                created_at,
                updated_at
            from user_webauthn_challenges where user_id = $1 and expires_at > now() and kind = $2 limit 1"#,
            id as UserId,
            kind as WebauthnChallengeKind
        )
        .fetch_one(pool)
        .await;

        match data {
            Ok(data) => Ok(Some(data)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e)?,
        }
    }
}
