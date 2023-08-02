use sqlx::FromRow;

use crate::prelude::*;
use crate::shared::DbPool;

#[derive(Debug, FromRow)]
pub struct Category {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
    pub ordering_index: i32,
}

impl Category {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO categories
                VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            self.id,
            self.user_id,
            self.name,
            self.description,
            self.hex_color,
            self.ordering_index
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn guard_one(pool: &DbPool, id: &str, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT id
                FROM categories
                WHERE id = $1 AND user_id = $2;
            "#,
            id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(())
    }
}
