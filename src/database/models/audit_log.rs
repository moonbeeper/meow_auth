use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::{error::DatabaseError, id::UlidId, models::user::UserId};

pub type AuditLogId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct AuditLog {
    #[builder(default = AuditLogId::new())]
    pub id: AuditLogId,
    pub user_id: UserId,
    pub action: String,
    #[builder(default = serde_json::json!({}))]
    pub metadata: serde_json::Value,
    #[builder(default = chrono::Utc::now())]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AuditLog {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into
                audit_logs (id, user_id, action, metadata, created_at)
             values
                ($1, $2, $3, $4, now())",
            self.id as AuditLogId,
            self.user_id as UserId,
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
}
