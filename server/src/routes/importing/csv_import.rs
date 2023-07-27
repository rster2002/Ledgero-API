use std::collections::HashMap;
use std::io::Cursor;

use chrono::Utc;
use csv::StringRecord;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::db_inner;
use crate::error::Error::Sqlx;
use crate::models::csv::csv_mapping::CsvImportOrdering::NewestFirst;
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use crate::models::entities::bank_account::BankAccount;
use crate::models::entities::import::Import;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::importing::map_csv_record::map_csv_record;
use crate::shared::{DbPool, SharedPool};
use crate::utils::try_collect::try_collect;

#[post("/csv", data = "<body>")]
pub async fn import_csv(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<ImportCsvDto>,
) -> Result<()> {
    let pool = db_inner!(pool);
    let body = body.0;

    // Start a database transaction.
    let mut db_transaction = pool.begin().await?;

    // Get required maps used when importing
    let mut bank_account_map = get_bank_accounts_map(pool, &user.uuid).await?;
    let external_account_map = get_external_accounts_map(pool, &user.uuid).await?;
    let mut order_indicator = get_order_indicator(pool, &user.uuid).await?;

    // Create an import record where all the transactions will be added to.
    let import_uuid = Uuid::new_v4();
    let import = Import {
        id: import_uuid.to_string(),
        user_id: user.uuid.to_string(),
        imported_at: Utc::now(),
        filename: body.filename,
    };

    // Create the parent import in the database
    import.create(&mut *db_transaction).await?;

    let records: Vec<StringRecord> =
        try_collect(csv::Reader::from_reader(Cursor::new(body.csv)).records())?;

    // If the first record is the newest, the order indicator should count down, so the indicator
    // is set to the highest value for the import (the number of transactions to import)
    if NewestFirst == body.mappings.ordering {
        order_indicator += records.len() as i32;
    }

    for record in records {
        let mapped_record = map_csv_record(record, &body.mappings)?;

        let bank_account_id: Result<String> = match bank_account_map
            .get(&*mapped_record.account_iban)
        {
            Some(id) => Ok(id.to_string()),
            None => {
                let bank_account = BankAccount {
                    id: Uuid::new_v4().to_string(),
                    iban: mapped_record.account_iban.to_string(),
                    user_id: user.uuid.to_string(),
                    name: mapped_record.account_iban.to_string(),
                    description: "A new bank account".to_string(),
                    hex_color: "ffffff".to_string(),
                };

                bank_account.create(&mut *db_transaction).await?;

                bank_account_map.insert(mapped_record.account_iban, bank_account.id.to_string());

                Ok(bank_account.id)
            }
        };

        if NewestFirst == body.mappings.ordering {
            order_indicator -= 1;
        } else {
            order_indicator += 1;
        }

        let mut transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            user_id: user.uuid.to_string(),
            transaction_type: TransactionType::Transaction,
            follow_number: mapped_record.follow_number,
            original_description: mapped_record.description.to_string(),
            description: mapped_record.description,
            complete_amount: mapped_record.amount,
            amount: mapped_record.amount,
            date: mapped_record.date,
            bank_account_id: Some(bank_account_id?),
            category_id: None,
            parent_transaction_id: None,
            external_account_name: mapped_record.external_account_name.to_string(),
            external_account_id: None,
            external_account_name_id: None,
            parent_import_id: Some(import_uuid.to_string()),
            subcategory_id: None,
            order_indicator,
            related_move_transaction: None,
        };

        let external_account_id = external_account_map
            .get(&*mapped_record.external_account_name)
            .map(|(id, category)| (id.to_string(), category.to_owned()));

        if let Some((external_id, category_id)) = external_account_id {
            transaction.external_account_id = Some(external_id);
            transaction.category_id = category_id;
        }

        // Because Postgres does an implicit rollback when a statement fails, a savepoint is created
        // so if the insert fails like we expect, the savepoint is the one that is implicitly
        // rolled back instead of the actual transaction.
        sqlx::query!("SAVEPOINT T")
            .execute(&mut *db_transaction)
            .await?;

        let result = transaction.create(&mut *db_transaction).await;

        // If the result is Ok the transactions is guaranteed to be a new transaction.
        if result.is_ok() {
            continue;
        }

        sqlx::query!("ROLLBACK TO T")
            .execute(&mut *db_transaction)
            .await?;

        // If the database returned an Err, the transaction may be a duplicate, so that is checked
        // here and if it is a duplicate, a link is created between the duplicate transaction and
        // the import record.
        let error = result.expect_err("Was Ok but also an error?");

        let Sqlx(wrapped_error) = &error else {
            return Err(error);
        };

        let Some(constraint) = wrapped_error.get_constraint() else {
            return Err(error);
        };

        // If the Err was not caused by the unique constrained the error import fails.
        if constraint != "unique_follow_number" {
            return Err(error);
        }

        sqlx::query!(
            r#"
                INSERT INTO SkippedTransactions
                VALUES ($1, $2, $3);
            "#,
            import_uuid.to_string(),
            user.uuid.to_string(),
            transaction.follow_number
        )
        .execute(&mut *db_transaction)
        .await?;
    }

    db_transaction.commit().await?;

    Ok(())
}

async fn get_bank_accounts_map(pool: &DbPool, user_id: &String) -> Result<HashMap<String, String>> {
    let records = sqlx::query!(
        r#"
            SELECT Id, IBAN
            FROM BankAccounts
            WHERE UserId = $1;
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::new();

    for record in records {
        map.insert(record.iban, record.id);
    }

    Ok(map)
}

async fn get_external_accounts_map(
    pool: &DbPool,
    user_id: &String,
) -> Result<HashMap<String, (String, Option<String>)>> {
    let records = sqlx::query!(
        r#"
            SELECT ExternalAccountNames.Name, ParentExternalAccount, e.DefaultCategoryId
            FROM ExternalAccountNames
            INNER JOIN ExternalAccounts e ON e.Id = ExternalAccountNames.ParentExternalAccount
            WHERE ExternalAccountNames.UserId = $1;
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::new();

    for record in records {
        map.insert(
            record.name,
            (record.parentexternalaccount, record.defaultcategoryid),
        );
    }

    Ok(map)
}

async fn get_order_indicator(pool: &DbPool, user_id: &String) -> Result<i32> {
    let record = sqlx::query!(
        r#"
            SELECT MAX(OrderIndicator) AS MaxIndicator
            FROM Transactions
            WHERE UserId = $1;
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(record.maxindicator.unwrap_or(0))
}
