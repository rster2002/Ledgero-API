use crate::prelude::*;
use crate::shared::DbPool;

pub struct Subcategory {
    pub id: String,
    pub user_id: String,
    pub parent_category: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}

impl Subcategory {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO Subcategories
                VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            self.id,
            self.user_id,
            self.parent_category,
            self.name,
            self.description,
            self.hex_color
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
