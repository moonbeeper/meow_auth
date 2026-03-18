use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

use crate::database::id::UlidId;

pub type HelloWorldId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
struct HelloWorld {
    #[builder(default = HelloWorldId::new())]
    pub id: HelloWorldId,
    #[builder(default = "meow".to_string())]
    pub message: String,
}

impl HelloWorld {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "insert into hello_world (id, message) values ($1, $2)",
            self.id as HelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            // message isn't unique but but for the sake of possible reuse in the future we use it here.
            "insert into hello_world (id, message) values ($1, $2) on conflict (message) do update set message = excluded.message",
            self.id as HelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), DatabaseError> {
        sqlx::query!(
            "update hello_world set message = $2 where id = $1",
            self.id as HelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        id: HelloWorldId,
        pool: &PgPool,
    ) -> Result<Option<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select * from hello_world where id = $1",
            id as HelloWorldId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_id(
        ids: Vec<HelloWorldId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, DatabaseError> {
        let data = sqlx::query_as!(
            Self,
            "select * from hello_world where id = ANY($1)",
            &ids as &[HelloWorldId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }
}
