use sqlx::FromRow;
use sqlx::{Executor, Postgres};

use crate::db_executor;
use crate::prelude::*;

/// A bank account something like a single IBAN. Used to differentiate between for example a savings
/// account and a 'regular' bank account.
#[derive(Debug, FromRow)]
pub struct BankAccount {
    pub id: String,
    pub iban: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}

impl BankAccount {
    pub async fn create<'d>(&self, executor: db_executor!('d)) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO BankAccounts
                VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            self.id,
            self.iban,
            self.user_id,
            self.name,
            self.description,
            self.hex_color
        )
        .execute(executor)
        .await?;

        Ok(())
    }
}
