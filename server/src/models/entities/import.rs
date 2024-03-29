use chrono::{DateTime, Utc};
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;
use sqlx::{Executor, Postgres};

use crate::db_executor;
use crate::prelude::*;
use crate::shared::DbPool;

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
    pub async fn create<'d>(&self, pool: db_executor!('d)) -> Result<()> {
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

    pub async fn guard_one(pool: &DbPool, id: &String, user_id: &String) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT id
                FROM imports
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
