use chrono::Utc;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Executor, FromRow, Postgres};

use crate::db_executor;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::prelude::*;
use crate::shared::DbPool;

pub mod transaction_type;

/// A single transaction of money.
#[derive(Debug, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct Transaction {
    pub id: String,
    pub user_id: String,

    pub transaction_type: TransactionType,

    /// A unique number for the transaction. This is different from the [id] in that this must be
    /// set when importing so that duplicate transaction are not accidentally imported twice.
    pub follow_number: String,

    /// The original description of the transaction. This should always match the description of
    /// the actual description of the transaction on the user's back account and should not be
    /// changed after creation.
    pub original_description: String,

    /// The description of the transaction. This is used to tell transactions apart from each-other.
    pub description: String,

    /// The actual amount of the complete transaction in euro cents, so 1,54 would be 156 in this
    /// field. The difference with [amount] is that amount may be changed by creating a split, while
    /// the complete amount should only be changed when there is an actual change to the transaction
    /// itself.
    pub complete_amount: i64,

    /// The current amount of the transaction. This may be changed by creating a split.
    pub amount: i64,

    /// Datetime of the transaction.
    pub date: chrono::DateTime<Utc>,

    /// The account id associated with the transaction.
    pub bank_account_id: String,

    /// The category this transaction belongs to. If the category id is [None] is is not part of
    /// a real category, but instead should be considered part of an "unsorted" category.
    pub category_id: Option<String>,

    /// The id of the parent transaction. This should be set for split transactions and should
    /// reference.
    pub parent_transaction_id: Option<String>,

    /// The name of the other account in this transaction. This is used to identify the external
    /// account, which may then be explicitly linked with the [external_account_id].
    pub external_account_name: String,

    /// The id referencing an external account entity. The [external_account_name] does not have
    /// to match with the actual name of the external account.
    pub external_account_id: Option<String>,

    /// The id of the parent [Import]. Used to group transactions that were created in an import.
    pub parent_import_id: Option<String>,

    pub subcategory_id: Option<String>,

    /// Indicator of the transaction order. Transactions should in general be ordered by the date
    /// of the transactions, but with for example the Rabobank CSV the export doesn't contain a time
    /// which could cause transaction to switch around when they're on the same date. This is used
    /// to give an indication of the correct order.
    pub order_indicator: i32,
}

impl Transaction {
    pub async fn create<'d>(&self, executor: db_executor!('d)) -> Result<()> {
        let transaction_type: &str = self.transaction_type.into();

        sqlx::query!(
            r#"
                INSERT INTO Transactions
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17);
            "#,
            self.id,
            self.user_id,
            Some(transaction_type),
            self.follow_number,
            self.description,
            self.original_description,
            self.complete_amount,
            self.amount,
            OffsetDateTime::from_unix_timestamp(self.date.timestamp())?,
            self.category_id,
            self.parent_transaction_id,
            self.external_account_name,
            self.external_account_id,
            self.bank_account_id,
            self.parent_import_id,
            self.subcategory_id,
            self.order_indicator
        )
        .execute(executor)
        .await?;

        Ok(())
    }

    pub async fn guard_one(pool: &DbPool, id: &str, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
                SELECT Id
                FROM Transactions
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
