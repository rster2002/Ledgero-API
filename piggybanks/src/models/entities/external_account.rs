use entity_macro::{Entity, table_name};

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
