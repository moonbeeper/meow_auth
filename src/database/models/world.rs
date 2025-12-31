use crate::database::ids::UlidId;
use sqlx::{PgPool, PgTransaction};
use typed_builder::TypedBuilder;

pub type DBHelloWorldId = UlidId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypedBuilder)]
pub struct DBHelloWorld {
    #[builder(default = DBHelloWorldId::new())]
    pub id: DBHelloWorldId,
    #[builder(default = "Hello world!!".to_string())]
    pub message: String,
}

// state of the art methods for db interactions that I WILL use again and again and again goddammit
impl DBHelloWorld {
    pub async fn insert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "insert into hello_world (id, message) values ($1, $2)",
            self.id as DBHelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            // message isn't unique but but for the sake of possible reuse in the future we use it here.
            "insert into hello_world (id, message) values ($1, $2) on conflict (message) do update set message = excluded.message",
            self.id as DBHelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn update(&self, transaction: &mut PgTransaction<'_>) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "update hello_world set message = $2 where id = $1",
            self.id as DBHelloWorldId,
            self.message
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    pub async fn find_by_id(
        &self,
        id: DBHelloWorldId,
        pool: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select * from hello_world where id = $1",
            id as DBHelloWorldId
        )
        .fetch_optional(pool)
        .await?;

        Ok(data)
    }

    pub async fn find_many_by_id(
        &self,
        ids: Vec<DBHelloWorldId>,
        pool: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let data = sqlx::query_as!(
            Self,
            "select * from hello_world where id = ANY($1)",
            &ids as &[DBHelloWorldId]
        )
        .fetch_all(pool)
        .await?;

        Ok(data)
    }
}
