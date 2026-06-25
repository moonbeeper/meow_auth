use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId};

pub type JwkKeyId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct JwkKey {
    #[builder(default = JwkKeyId::new())]
    pub id: JwkKeyId,
    pub secret: Vec<u8>,
    pub nonce: Vec<u8>,
    #[builder(default = false)]
    pub retired: bool,
    pub retired_at: chrono::DateTime<chrono::Utc>,
    pub max_public_age_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl JwkKey {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into jwks_keys (
                id,
                secret,
                nonce,
                retired,
                retired_at,
                max_public_age_at
            )
            values ($1, $2, $3, $4, $5, $6)
            ",
            self.id as JwkKeyId,
            self.secret,
            self.nonce,
            self.retired,
            self.retired_at,
            self.max_public_age_at
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    // pub async fn set_retired(
    //     &self,
    //     transaction: &mut PgTransaction<'_>,
    // ) -> Result<(), DatabaseError> {
    //     sqlx::query!(
    //         "update jwks_keys set
    //             retired = true,
    //             updated_at = now()
    //          where id = $1",
    //         self.id as JwkKeyId,
    //     )
    //     .execute(&mut **transaction)
    //     .await?;

    //     Ok(())
    // }

    pub async fn delete_non_public(
        &self,
        transaction: &mut PgTransaction<'_>,
    ) -> Result<(), DatabaseError> {
        sqlx::query!("delete from jwks_keys where max_public_age_at < NOW()")
            .execute(&mut **transaction)
            .await?;

        Ok(())
    }

    pub async fn get_active(pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                secret,
                nonce,
                retired,
                retired_at,
                max_public_age_at,
                updated_at,
                created_at
             from jwks_keys where not retired order by created_at limit 1",
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    // pub async fn get_public(pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
    //     let data = sqlx::query_as!(
    //         Self,
    //         "select
    //             id,
    //             secret,
    //             nonce,
    //             retired,
    //             retired_at,
    //             max_public_age_at,
    //             updated_at,
    //             created_at
    //          from jwks_keys where not retired and max_public_age_at > now() order by created_at",
    //     )
    //     .fetch_optional(pool)
    //     .await?;

    //     Ok(data)
    // }

    pub async fn get_retired(pool: &PgPool) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                secret,
                nonce,
                retired,
                retired_at,
                max_public_age_at,
                updated_at,
                created_at
             from jwks_keys where retired and max_public_age_at >= now() order by created_at",
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn set_retire(transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update jwks_keys set
                retired = true,
                updated_at = now()
            where not retired and id != (
                select id from jwks_keys where not retired order by created_at desc limit 1
            )
            "
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
