use sqlx::FromRow;
use entity_macro::{Entity, table_name};
use crate::shared_types::DbPool;
use crate::prelude::*;

/// Used to link a name in the [Transaction::external_account_name] to an actual [ExternalAccount].
#[derive(Debug, FromRow, Entity)]
#[table_name("ExternalAccountNames")]
#[sqlx(rename_all = "PascalCase")]
pub struct ExternalAccountName {
    pub id: String,
    pub user_id: String,

    /// The name of the external account. This is what is matched with
    /// [Transaction::external_account_name] to check if the transaction is part of an external
    /// account.
    pub name: String,
    pub parent_external_account: String,
}

impl ExternalAccountName {
    pub async fn guard_one(pool: &DbPool, id: &String, user_id: &String) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT Id
                FROM ExternalAccountNames
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
