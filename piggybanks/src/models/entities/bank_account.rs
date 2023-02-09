use entity_macro::{table_name, Entity};
use sqlx::FromRow;

/// A bank account something like a single IBAN. Used to differentiate between for example a savings
/// account and a 'regular' bank account.
#[derive(Debug, FromRow, Entity)]
#[table_name("BankAccounts")]
#[sqlx(rename_all = "PascalCase")]
pub struct BankAccount {
    pub id: String,
    pub iban: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}
