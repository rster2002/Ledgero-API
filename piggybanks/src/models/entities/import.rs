use chrono::{DateTime, Utc};
use sqlx::FromRow;
use sqlx::types::time::OffsetDateTime;
use crate::shared_types::DbPool;
use crate::prelude::*;

#[derive(Debug, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct Import {
    pub id: String,
    pub user_id: String,

    /// The time at which the import was performed.
    pub imported_at: DateTime<Utc>,

    /// The name of the file used to create this import.
    pub filename: String,
}

impl Import {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO Imports
                VALUES ($1, $2, $3, $4);
            "#,
            self.id,
            self.user_id,
            OffsetDateTime::from_unix_timestamp(self.imported_at.timestamp())?,
            self.filename
        )
            .execute(pool)
            .await?;

        Ok(())
    }
}
