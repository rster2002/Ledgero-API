use std::collections::HashMap;
use std::io::Cursor;
use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::error::Error::Sqlx;
use crate::error::import_error::ImportError;
use crate::models::csv::csv_mapping::{AmountMapping, DateMapping};
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use crate::models::entities::bank_account::BankAccount;
use crate::models::entities::import::Import;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::shared_types::{DbPool, SharedPool};
use crate::prelude::*;
use crate::routes::importing::map_csv_record::map_csv_record;

#[post("/csv", data = "<body>")]
pub async fn import_csv(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<ImportCsvDto>,
) -> Result<()> {
    let body = body.0;

    // Start a database transaction.
    let db_transaction = pool.begin().await?;

    let mut bank_account_map = get_bank_accounts_map(pool, &user.uuid).await?;
    let external_account_map = get_external_accounts_map(pool, &user.uuid).await?;

    // Create an import record where all the transactions will be added to.
    let import_uuid = Uuid::new_v4();
    let import = Import {
        id: import_uuid.to_string(),
        user_id: user.uuid.to_string(),
        imported_at: Utc::now(),
        filename: body.filename,
    };

    import.create(pool)
        .await?;

    let mut reader = csv::Reader::from_reader(Cursor::new(body.csv));

    for record in reader.records() {
        let mapped_record = map_csv_record(record?, &body.mappings)?;

        let bank_account_id: Result<String> = match bank_account_map.get(&*mapped_record.account_iban) {
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

                bank_account.create(pool).await?;

                bank_account_map.insert(mapped_record.account_iban, bank_account.id.to_string());

                Ok(bank_account.id)
            }
        };

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
            bank_account_id: bank_account_id?,
            category_id: None,
            parent_transaction_id: None,
            external_account_name: mapped_record.external_account_name.to_string(),
            external_account_id: None,
            parent_import_id: Some(import_uuid.to_string())
        };

        let external_account_id = external_account_map
            .get(&*mapped_record.external_account_name)
            .map(|(id, category)| (id.to_string(), category.to_owned()));

        if let Some((external_id, category_id)) = external_account_id {
            transaction.external_account_id = Some(external_id);
            transaction.category_id = category_id;
        }

        let result = transaction.create(pool).await;

        // If the result is Ok the transactions is guaranteed to be a new transaction.
        if let Ok(_) = result {
            continue;
        }

        // If the database returned an Err, the transaction may be a duplicate, so that is checked
        // here and if it is a duplicate, a link is created between the duplicate transaction and
        // the import record.
        let error = result
            .expect_err("Was Ok but also an error?");

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
            .execute(pool.inner())
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
