use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{
    error::DatabaseError,
    id::UlidId,
    models::user::UserId,
    pagination::{PaginatedId, PaginationResult},
};

pub type AuditLogId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct AuditLog {
    #[builder(default = AuditLogId::new())]
    pub id: AuditLogId,
    pub user_id: UserId,
    pub actor_id: UserId,
    pub action: String,
    #[builder(default = serde_json::json!({}))]
    pub metadata: serde_json::Value,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// i hate you. yes you, mr space birb that hecking codes as shit and makes really bad decisions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditLogLogin {
    pub id: AuditLogId,
    pub user_id: UserId,
    pub user_login: String,
    pub actor_id: UserId,
    pub actor_login: String,
    pub action: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AuditLog {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                audit_logs (id, user_id, actor_id, action, metadata, created_at)
             values
                ($1, $2, $3, $4, $5, now())",
            self.id as AuditLogId,
            self.user_id as UserId,
            self.actor_id as UserId,
            self.action,
            self.metadata,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_user(id: UserId, pool: &PgPool) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select
                id,
                user_id,
                actor_id,
                action,
                metadata,
                created_at
             from audit_logs where user_id = $1",
            id as UserId
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_by_user_with_logins(
        user_id: UserId,
        pool: &PgPool,
    ) -> Result<Vec<AuditLogLogin>, DatabaseError> {
        let data = sqlx::query_as!(
            AuditLogLogin,
            r#"select
                al.id,
                al.user_id,
                u.login as user_login,
                al.actor_id,
                a.login as actor_login,
                al.action,
                al.metadata,
                al.created_at
            from audit_logs al
            join users u on u.id = al.user_id
            join users a on a.id = al.actor_id
            where user_id = $1
            "#,
            user_id as UserId
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_user_id_with_logins_paginated(
        user_id: UserId,
        from: Option<AuditLogId>,
        want_total: bool,
        pool: &PgPool,
    ) -> Result<PaginationResult<AuditLogLogin>, DatabaseError> {
        let data = sqlx::query_as!(
            AuditLogLogin,
            r#"select
                al.id,
                al.user_id,
                u.login as user_login,
                al.actor_id,
                a.login as actor_login,
                al.action,
                al.metadata,
                al.created_at
            from audit_logs al
            join users u on u.id = al.user_id
            join users a on a.id = al.actor_id
            where al.user_id = $1 and ($2::uuid is null or al.id::uuid > $2) order by created_at asc limit 20+1
            "#,
            user_id as UserId,
            from as Option<AuditLogId>
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

    pub async fn find_many_by_user_id_paginated(
        user_id: UserId,
        from: Option<AuditLogId>,
        want_total: bool,
        pool: &PgPool,
    ) -> Result<PaginationResult<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            r#"select
                id,
                user_id,
                actor_id,
                action,
                metadata,
                created_at
            from audit_logs where user_id = $1 and ($2::uuid is null or id::uuid > $2) order by created_at asc limit 20+1
            "#,
            user_id as UserId,
            from as Option<AuditLogId>
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
            "select count(*) from audit_logs where user_id = $1",
            user_id as UserId,
        )
        .fetch_one(pool)
        .await?;

        Ok(data)
    }
}

impl PaginatedId for AuditLogLogin {
    fn paginated_id(&self) -> UlidId {
        self.id
    }
}

impl PaginatedId for AuditLog {
    fn paginated_id(&self) -> UlidId {
        self.id
    }
}
