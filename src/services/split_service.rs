use chrono::Utc;
use sqlx::{Postgres};
use uuid::Uuid;

use crate::error::http_error::HttpError;
use crate::models::dto::transactions::new_split_dto::NewSplitDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::prelude::*;
use crate::shared::DbTransaction;

pub struct SplitService;

impl SplitService {
    pub async fn create_split(
        mut db_transaction: DbTransaction<'_>,
        user_id: String,
        transaction_id: String,
        body: NewSplitDto,
    ) -> Result<sqlx::Transaction<'_, Postgres>> {
        let parent_transaction = sqlx::query!(
            r#"
                SELECT Id, BankAccountId, Amount, ExternalAccountName, ExternalAccountId
                FROM Transactions
                WHERE Id = $1 AND UserId = $2;
            "#,
            transaction_id,
            user_id
        )
        .fetch_one(&mut db_transaction)
        .await?;

        Self::guard_amount(parent_transaction.amount, body.amount)?;

        let split_transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            transaction_type: TransactionType::Split,
            follow_number: Uuid::new_v4().to_string(),
            original_description: body.description.to_string(),
            description: body.description,
            complete_amount: body.amount,
            amount: body.amount,
            date: Utc::now(),
            bank_account_id: parent_transaction.bankaccountid,
            category_id: body.category_id,
            parent_transaction_id: Some(parent_transaction.id),
            external_account_name: parent_transaction.externalaccountname,
            external_account_id: parent_transaction.externalaccountid,
            parent_import_id: None,
            subcategory_id: body.subcategory_id,
            order_indicator: 0,
        };

        split_transaction.create(&mut db_transaction).await?;

        let new_amount = parent_transaction.amount - split_transaction.amount;

        sqlx::query!(
            r#"
                UPDATE Transactions
                SET Amount = $3
                WHERE Id = $1 AND UserId = $2;
            "#,
            transaction_id,
            user_id,
            new_amount
        )
        .execute(&mut db_transaction)
        .await?;

        Ok(db_transaction)
    }

    pub async fn update_split(
        mut db_transaction: DbTransaction<'_>,
        user_id: String,
        transaction_id: String,
        split_id: String,
        body: NewSplitDto,
    ) -> Result<sqlx::Transaction<'_, Postgres>> {
        let parent_transaction = sqlx::query!(
            r#"
                SELECT Id, BankAccountId, Amount, ExternalAccountName, ExternalAccountId
                FROM Transactions
                WHERE Id = $1 AND UserId = $2;
            "#,
            transaction_id,
            user_id
        )
        .fetch_one(&mut db_transaction)
        .await?;

        let split = sqlx::query!(
            r#"
                SELECT Id, Amount
                FROM Transactions
                WHERE
                    TransactionType = 'split' AND
                    UserId = $1 AND
                    ParentTransactionId = $2 AND
                    Id = $3;
            "#,
            user_id,
            transaction_id,
            split_id
        )
        .fetch_one(&mut db_transaction)
        .await?;

        let available_amount = parent_transaction.amount + split.amount;
        SplitService::guard_amount(available_amount, body.amount)?;

        let new_parent_amount = available_amount - body.amount;

        sqlx::query!(
            r#"
                UPDATE Transactions
                SET Description = $3, Amount = $4, CategoryId = $5
                WHERE Id = $1 AND UserId = $2;
            "#,
            split_id,
            user_id,
            body.description,
            body.amount,
            body.category_id
        )
        .execute(&mut db_transaction)
        .await?;

        sqlx::query!(
            r#"
                UPDATE Transactions
                SET Amount = $3
                WHERE Id = $1 AND UserId = $2;
            "#,
            transaction_id,
            user_id,
            new_parent_amount
        )
        .execute(&mut db_transaction)
        .await?;

        Ok(db_transaction)
    }

    fn guard_amount(parent_amount: i64, split_amount: i64) -> Result<()> {
        if (parent_amount > 0 && split_amount > parent_amount)
            || (parent_amount < 0 && split_amount < parent_amount)
        {
            return Err(
                HttpError::new(400)
                    .message("Cannot create a split with an amount bigger than the remaining about of the parent")
                    .into()
            );
        }

        Ok(())
    }
}
