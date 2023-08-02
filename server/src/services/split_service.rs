use chrono::Utc;
use sqlx::Postgres;
use uuid::Uuid;

use crate::error::http_error::HttpError;
use crate::models::dto::transactions::new_split_dto::NewSplitDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::prelude::*;
use crate::shared::DbTransaction;

pub struct SplitService;

impl SplitService {
    pub async fn create_split<'a>(
        db_transaction: &mut DbTransaction<'a>,
        user_id: &'a str,
        transaction_id: &'a str,
        body: NewSplitDto<'a>,
    ) -> Result<()> {
        trace!("Creating new split");

        trace!("Fetching parent transaction from database");
        let parent_transaction = sqlx::query!(
            r#"
                SELECT id, bank_account_id, amount, external_account_name, external_account_id
                FROM transactions
                WHERE id = $1 AND user_id = $2;
            "#,
            transaction_id,
            user_id
        )
        .fetch_one(&mut **db_transaction)
        .await?;

        let split_amount: i64 = if parent_transaction.amount < 0 {
            -(body.amount as i64)
        } else {
            body.amount as i64
        };

        Self::guard_amount(parent_transaction.amount, split_amount)?;

        let split_transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            transaction_type: TransactionType::Split,
            follow_number: Uuid::new_v4().to_string(),
            original_description: body.description.to_string(),
            description: body.description.to_string(),
            complete_amount: split_amount,
            amount: split_amount,
            date: Utc::now(),
            bank_account_id: parent_transaction.bank_account_id,
            category_id: body.category_id.map(|v| v.to_string()),
            subcategory_id: body.subcategory_id.map(|v| v.to_string()),
            parent_transaction_id: Some(parent_transaction.id),
            external_account_name: parent_transaction.external_account_name,
            external_account_id: parent_transaction.external_account_id,
            external_account_name_id: None,
            parent_import_id: None,
            order_indicator: 0,
            related_move_transaction: None,
        };

        debug!("Creating new split with id '{}'", split_transaction.id);
        split_transaction.create(&mut **db_transaction).await?;

        let new_amount = parent_transaction.amount - split_transaction.amount;

        trace!("Updating parent transaction with new remainder");
        sqlx::query!(
            r#"
                UPDATE transactions
                SET amount = $3
                WHERE id = $1 AND user_id = $2;
            "#,
            transaction_id,
            user_id,
            new_amount
        )
        .execute(&mut **db_transaction)
        .await?;

        Ok(())
    }

    pub async fn update_split<'a>(
        db_transaction: &mut DbTransaction<'a>,
        user_id: &'a str,
        transaction_id: &'a str,
        split_id: &'a str,
        body: NewSplitDto<'a>,
    ) -> Result<()> {
        let parent_transaction = sqlx::query!(
            r#"
                SELECT id, bank_account_id, amount, external_account_name, external_account_id
                FROM transactions
                WHERE id = $1 AND user_id = $2;
            "#,
            transaction_id,
            user_id
        )
        .fetch_one(&mut **db_transaction)
        .await?;

        let split = sqlx::query!(
            r#"
                SELECT id, amount
                FROM transactions
                WHERE
                    transaction_type = 'split' AND
                    user_id = $1 AND
                    parent_transaction_id = $2 AND
                    id = $3;
            "#,
            user_id,
            transaction_id,
            split_id
        )
        .fetch_one(&mut **db_transaction)
        .await?;

        let split_amount = if parent_transaction.amount < 0 {
            -(body.amount as i64)
        } else {
            body.amount as i64
        };

        let available_amount = parent_transaction.amount + split.amount;
        SplitService::guard_amount(available_amount, split_amount)?;

        let new_parent_amount = available_amount - split_amount;

        sqlx::query!(
            r#"
                UPDATE transactions
                SET description = $3, amount = $4, category_id = $5
                WHERE id = $1 AND user_id = $2;
            "#,
            split_id,
            user_id,
            body.description,
            split_amount,
            body.category_id
        )
        .execute(&mut **db_transaction)
        .await?;

        sqlx::query!(
            r#"
                UPDATE transactions
                SET amount = $3
                WHERE id = $1 AND user_id = $2;
            "#,
            transaction_id,
            user_id,
            new_parent_amount
        )
        .execute(&mut **db_transaction)
        .await?;

        Ok(())
    }

    fn guard_amount(available_amount: i64, split_amount: i64) -> Result<()> {
        if !SplitService::check_amount(available_amount, split_amount) {
            trace!("Split amount exceeds available amount");
            return Err(
                HttpError::new(400)
                    .message("Cannot create a split with an amount bigger than the remaining about of the parent")
                    .into()
            );
        }

        Ok(())
    }

    fn check_amount(available_amount: i64, split_amount: i64) -> bool {
        if available_amount >= 0 {
            split_amount <= available_amount && split_amount > 0
        } else {
            split_amount >= available_amount && split_amount < 0
        }
    }
}
