use sqlx::FromRow;
use entity_macro::{Entity, table_name};
use crate::prelude::*;
use crate::shared_types::DbPool;

#[derive(Debug, FromRow, Entity)]
#[table_name("Categories")]
#[sqlx(rename_all = "PascalCase")]
pub struct Category {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}

impl Category {
    pub async fn guard_one(pool: &DbPool, id: &String, user_id: &String) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT Id
                FROM Categories
                WHERE Id = $1 AND UserId = $2;
            "#,
            id,
            user_id
        )
            .fetch_one(pool)
            .await?;

        Ok(())
    }
}
