use sqlx::FromRow;
use crate::prelude::*;
use crate::shared_types::DbPool;

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
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
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
            .execute(pool)
            .await?;

        Ok(())
    }
}
