use sqlx::FromRow;

use crate::prelude::*;
use crate::shared::DbPool;

/// An external account is used to group transactions to another party of the transaction, like for
/// example a super market.
#[derive(Debug, FromRow)]
pub struct ExternalAccount {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,

    /// If this is set, when a transaction is matched an external account, the category id is
    /// automatically set to this value.
    pub default_category_id: Option<String>,
    pub default_subcategory_id: Option<String>,
}

impl ExternalAccount {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO external_accounts
                VALUES ($1, $2, $3, $4, $5, $6, null, $7);
            "#,
            self.id,
            self.user_id,
            self.name,
            self.description,
            self.default_category_id,
            self.default_subcategory_id,
            self.hex_color,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn guard_one(pool: &DbPool, id: &String, user_id: &String) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT id
                FROM external_accounts
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
