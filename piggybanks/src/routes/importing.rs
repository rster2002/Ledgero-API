use std::collections::{BTreeMap, HashMap};
use std::io::Cursor;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use csv::StringRecord;
use rocket::figment::map;
use rocket::Route;
use rocket::serde::json::Json;
use rocket::time::macros::date;
use uuid::Uuid;
use crate::error::import_error::ImportError;
use crate::models::csv::csv_mapping::{AmountMapping, DateMapping};
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use crate::models::entities::bank_account::BankAccount;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::{DbPool, SharedPool};

pub fn create_importing_routes() -> Vec<Route> {
    routes![
        import_csv,
    ]
}

#[post("/csv", data="<body>")]
pub async fn import_csv(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<ImportCsvDto>,
) -> Result<()> {
    let body = body.0;
    let mappings = body.mappings;

    // Start a database transaction
    let db_transaction = pool.begin().await?;

    let mut bank_account_map = get_bank_accounts_map(pool, &user.uuid)
        .await?;

    let mut external_account_map = get_external_accounts_map(pool, &user.uuid)
        .await?;

    let mut reader = csv::Reader::from_reader(Cursor::new(body.csv));

    for record in reader.records() {
        let record = record?;

        let follow_number = record.get(mappings.follow_number as usize)
            .ok_or(ImportError::missing_column("follow_number"))?
            .to_string();

        let description = record.get(mappings.description as usize)
            .ok_or(ImportError::missing_column("description"))?
            .to_string();

        let temp_amount = record.get(mappings.amount as usize)
            .ok_or(ImportError::missing_column("amount"))?
            .replace('+', "")
            .replace(',', ".")
            .parse::<f64>()?;

        let amount: i64 = match mappings.amount_mapping {
            AmountMapping::Cents => temp_amount as i64,
            AmountMapping::Euro => (temp_amount * 100_f64) as i64
        };

        let date_string = record.get(mappings.date as usize)
            .ok_or(ImportError::missing_column("date"))?
            .to_string();

        let date = map_datetime(&date_string, &mappings.date_mapping)?;

        let bank_account_iban = record.get(mappings.account_iban as usize)
            .ok_or(ImportError::missing_column("iban"))?
            .to_string();

        let bank_account_id: Result<String> = match bank_account_map.get(&*bank_account_iban) {
            Some(id) => Ok(id.to_string()),
            None => {
                let bank_account = BankAccount {
                    id: Uuid::new_v4().to_string(),
                    iban: bank_account_iban.to_string(),
                    user_id: user.uuid.to_string(),
                    name: bank_account_iban.to_string(),
                    description: "A new bank account".to_string(),
                    hex_color: "ffffff".to_string(),
                };

                bank_account.create(pool)
                    .await?;

                bank_account_map.insert(bank_account_iban, bank_account.id.to_string());

                Ok(bank_account.id)
            }
        };

        let external_account_name = record.get(mappings.external_account_name as usize)
            .ok_or("Column for external_account_name does not exist")?
            .to_string();

        let external_account_id = external_account_map.get(&*external_account_name)
            .map(|x| x.to_string());

        let transaction = Transaction {
            id: Uuid::new_v4().to_string(),
            user_id: user.uuid.to_string(),
            transaction_type: TransactionType::Transaction,
            follow_number,
            original_description: description.to_string(),
            description,
            complete_amount: amount,
            amount,
            date,
            bank_account_id: bank_account_id?,
            category_id: None,
            parent_transaction_id: None,
            external_account_name,
            external_account_id,
        };

        transaction.create(pool)
            .await?;
    }

    db_transaction.commit()
        .await?;

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

async fn get_external_accounts_map(pool: &DbPool, user_id: &String) -> Result<HashMap<String, String>> {
    let records = sqlx::query!(
        r#"
            SELECT Name, ParentExternalAccount
            FROM ExternalAccountNames
            WHERE UserId = $1;
        "#,
        user_id
    )
        .fetch_all(pool)
        .await?;

    let mut map = HashMap::new();

    for record in records {
        map.insert(record.name, record.parentexternalaccount);
    }

    Ok(map)
}

fn map_datetime(col_value: &String, date_mapping: &DateMapping) -> Result<String> {
    let mut working_value = col_value.to_string();

    if let Some(template) = &date_mapping.template {
        working_value = template.replace('$', &*working_value);
    }

    let datetime = DateTime::parse_from_str(&working_value, &date_mapping.format)?;

    Ok(datetime.to_string())
}
