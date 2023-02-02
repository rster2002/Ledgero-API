use sqlx::FromRow;
use entity_macro::{Entity, table_name};

/// Used to link a name in the [Transaction::external_account_name] to an actual [ExternalAccount].
#[derive(Debug, FromRow, Entity)]
#[table_name("ExternalAccountNames")]
#[sqlx(rename_all = "PascalCase")]
pub struct ExternalAccountNames {
    pub id: String,
    pub user_id: String,

    /// The name of the external account. This is what is matched with
    /// [Transaction::external_account_name] to check if the transaction is part of an external
    /// account.
    pub name: String,
    pub parent_external_account: String,
}
