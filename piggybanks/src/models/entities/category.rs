use crate::prelude::*;
use crate::shared_types::DbPool;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Category {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}

impl Category {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO Categories
                VALUES ($1, $2, $3, $4, $5);
            "#,
            self.id,
            self.user_id,
            self.name,
            self.description,
            self.hex_color
        )
            .execute(pool)
            .await?;

        Ok(())
    }

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
