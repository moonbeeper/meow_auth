use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    pagination::{PaginatedId, PaginationResult},
};

pub type UserId = UlidId;
pub type PIDUserId = UlidId;

// TODO: add pid to user for external clients or providers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct User {
    #[builder(default = UserId::new())]
    pub id: UserId,
    #[builder(default = PIDUserId::new())]
    pub pid: PIDUserId,
    pub name: String,
    pub email: String,
    #[builder(default = false)]
    pub email_verified: bool,
    #[builder(default = false)]
    pub totp_enabled: bool,
    #[builder(default = false)]
    pub has_webauthn: bool,
    #[builder(default = 0)]
    pub flags: i64,
    #[builder(default = chrono::Utc::now())]
    pub name_updated_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[builder(default = chrono::Utc::now())]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                users (id, pid, name, email, email_verified, flags, name_updated_at ,created_at, updated_at)
             values
                ($1, $2, lower($3), lower($4), $5, $6, now(), now(), now())",
            self.id as UserId,
            self.pid as PIDUserId,
            self.name,
            self.email,
            self.email_verified,
            self.flags
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update users set
                name = $2,
                email = $3,
                email_verified = $4,
                totp_enabled = $5,
                has_webauthn = $6,
                name_updated_at = $7,
                flags = $8,
                updated_at = now()
             where id = $1",
            self.id as UserId,
            self.name,
            self.email,
            self.email_verified,
            self.totp_enabled,
            self.has_webauthn,
            self.name_updated_at,
            self.flags
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
                pid,
                name,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                flags,
                name_updated_at,
                created_at,
                updated_at
             from users where id = $1",
            id as UserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_pid(pid: PIDUserId, pool: &PgPool) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                name,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                flags,
                name_updated_at,
                created_at,
                updated_at
             from users where pid = $1",
            pid as PIDUserId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_email(
        email: String,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                name,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                flags,
                name_updated_at,
                created_at,
                updated_at
             from users where email = lower($1)",
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    // pub async fn find_by_login(
    //     login: String,
    //     pool: &PgPool,
    // ) -> Result<Option<Self>, DatabaseError> {
    //     let data = sqlx::query_as!(
    //         Self,
    //         "select
    //             id,
    //             pid,
    //             login,
    //             email,
    //             email_verified,
    //             totp_enabled,
    //             has_webauthn,
    //             flags,
    //             login_updated_at,
    //             created_at,
    //             updated_at
    //          from users where login = lower($1)",
    //         login
    //     )
    //     .fetch_optional(pool)
    //     .await?;

    //     Ok(data)
    // }

    pub async fn find_many_by_id(
        ids: Vec<UserId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                name,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                flags,
                name_updated_at,
                created_at,
                updated_at
             from users where id = any($1)",
            &ids as &[UserId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_paginated(
        from: Option<UserId>,
        want_total: bool,
        pool: &PgPool,
    ) -> Result<PaginationResult<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                pid,
                name,
                email,
                email_verified,
                totp_enabled,
                has_webauthn,
                flags,
                name_updated_at,
                created_at,
                updated_at
             from users where ($1::uuid is null or id::uuid > $1) order by created_at asc limit 20+1",
            from as Option<UserId>
        )
        .fetch_all(pool)
        .await?;

        let total_rows = if want_total {
            Self::count_all(pool).await?
        } else {
            None
        };

        Ok(PaginationResult::new(data, total_rows))
    }

    pub async fn get_flags_by_id(id: UserId, pool: &PgPool) -> Result<Option<i64>, DatabaseError> {
        let data = sqlx::query_scalar!("select flags from users where id = $1", id as UserId)
            .fetch_optional(pool)
            .await?;

        Ok(data)
    }

    pub async fn count_all(pool: &PgPool) -> Result<Option<i64>, DatabaseError> {
        let data = sqlx::query_scalar!("select count(*) from users")
            .fetch_one(pool)
            .await?;

        Ok(data)
    }
}

impl PaginatedId for User {
    fn paginated_id(&self) -> UlidId {
        self.id
    }
}
