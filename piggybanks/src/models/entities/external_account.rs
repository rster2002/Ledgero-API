use sqlx::FromRow;
use entity_macro::{Entity, table_name};
use crate::shared_types::DbPool;
use crate::prelude::*;

/// An external account is used to group transactions to another party of the transaction, like for
/// example a super market.
#[derive(Debug, FromRow, Entity)]
#[table_name("ExternalAccounts")]
#[sqlx(rename_all = "PascalCase")]
pub struct ExternalAccount {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,

    /// If this is set, when a transaction is matched an external account, the category id is
    /// automatically set to this value.
    pub default_category_id: Option<String>,
}

impl ExternalAccount {
    pub async fn guard_one(pool: &DbPool, id: &String, user_id: &String) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT Id
                FROM ExternalAccounts
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
