use std::collections::BTreeMap;
use rocket::serde::json::Json;
use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use crate::models::entities::category::Category;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::shared_types::SharedPool;
use crate::prelude::*;

pub struct TransactionRecord {
    pub transactionid: String,
    pub transactiontype: String,
    pub follownumber: String,
    pub originaldescription: String,
    pub description: String,
    pub completeamount: i64,
    pub amount: i64,
    pub bankaccountid: String,
    pub bankaccountiban: String,
    pub bankaccountname: String,
    pub bankaccountdescription: String,
    pub bankaccounthexcolor: String,
    pub externalaccountname: String,
    pub CategoryId: Option<String>,
    pub CategoryName: Option<String>,
    pub CategoryDescription: Option<String>,
    pub CategoryHexColor: Option<String>,
    pub ExternalAccountId: Option<String>,
    pub ExternalAccountEntityName: Option<String>,
    pub ExternalAccountDescription: Option<String>,
    pub ExternalAccounDefaultCategoryId: Option<String>,
}

#[get("/")]
pub async fn get_all_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<TransactionDto>>> {
    let pool = pool.inner();

    let records = sqlx::query_as!(
        TransactionRecord,
        r#"
            SELECT
                transactions.Id as TransactionId, TransactionType, FollowNumber, OriginalDescription, transactions.Description, CompleteAmount, Amount, ExternalAccountName,
                c.Id as "CategoryId?", c.Name as "CategoryName?", c.Description as "CategoryDescription?", c.HexColor as "CategoryHexColor?",
                b.Id as BankAccountId, b.Iban as BankAccountIban, b.Name as BankAccountName, b.Description as BankAccountDescription, b.HexColor as BankAccountHexColor,
                e.Id as "ExternalAccountId?", e.Name as "ExternalAccountEntityName?", e.Description as "ExternalAccountDescription?", e.DefaultCategoryId as "ExternalAccounDefaultCategoryId?"
            FROM Transactions
            LEFT JOIN categories c on transactions.categoryid = c.id
            LEFT JOIN bankaccounts b on transactions.bankaccountid = b.id
            LEFT JOIN externalaccounts e on c.id = e.defaultcategoryid
            WHERE TransactionType = 'transaction' AND Transactions.UserId = $1;
        "#,
        user.uuid
    )
        .fetch_all(pool)
        .await?;

    let transactions = records
        .into_iter()
        .map(|record| map_record(record))
        .collect();

    Ok(Json(transactions))
}

#[get("/<id>")]
pub async fn get_single_transaction(
    id: String,
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<TransactionDto>> {
    let pool = pool.inner();

    let record = sqlx::query_as!(
        TransactionRecord,
        r#"
            SELECT
                transactions.Id as TransactionId, TransactionType, FollowNumber, OriginalDescription, transactions.Description, CompleteAmount, Amount, ExternalAccountName,
                c.Id as "CategoryId?", c.Name as "CategoryName?", c.Description as "CategoryDescription?", c.HexColor as "CategoryHexColor?",
                b.Id as BankAccountId, b.Iban as BankAccountIban, b.Name as BankAccountName, b.Description as BankAccountDescription, b.HexColor as BankAccountHexColor,
                e.Id as "ExternalAccountId?", e.Name as "ExternalAccountEntityName?", e.Description as "ExternalAccountDescription?", e.DefaultCategoryId as "ExternalAccounDefaultCategoryId?"
            FROM Transactions
            LEFT JOIN categories c on transactions.categoryid = c.id
            LEFT JOIN bankaccounts b on transactions.bankaccountid = b.id
            LEFT JOIN externalaccounts e on c.id = e.defaultcategoryid
            WHERE TransactionType = 'transaction' AND Transactions.UserId = $1 AND Transactions.Id = $2;
        "#,
        user.uuid,
        id
    )
        .fetch_one(pool)
        .await?;

    let transaction = map_record(record);

    Ok(Json(transaction))
}

#[patch("/<id>/category", data="<body>")]
pub async fn change_category_for_transaction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<TransactionSetCategoryDto>,
) -> Result<()> {
    let pool = pool.inner();
    let body = body.0;

    Transaction::guard_one(pool, &id, &user.uuid)
        .await?;

    if let Some(category_id) = &body.category_id {
        Category::guard_one(pool, &category_id, &user.uuid)
            .await?;
    }

    sqlx::query!(
        r#"
            UPDATE Transactions
            SET CategoryId = $3
            WHERE Id = $1 AND UserId = $2
        "#,
        id,
        user.uuid,
        body.category_id
    )
        .execute(pool)
        .await?;

    Ok(())
}

pub fn map_record(record: TransactionRecord) -> TransactionDto {
    let mut transaction = TransactionDto {
        id: record.transactionid,
        transaction_type: TransactionType::from(&*record.transactiontype),
        follow_number: record.follownumber,
        original_description: record.originaldescription,
        description: record.description,
        complete_amount: record.completeamount,
        amount: record.amount,
        bank_account: BankAccountDto {
            id: record.bankaccountid,
            iban: record.bankaccountiban,
            name: record.bankaccountname,
            description: record.bankaccountdescription,
            hex_color: record.bankaccounthexcolor,
        },
        category: None,
        external_account_name: record.externalaccountname,
        external_account: None,
    };

    if let Some(id) = record.CategoryId {
        transaction.category = Some(CategoryDto {
            id,
            name: record.CategoryName
                .expect("Category id was not null, but the category name was"),
            description: record.CategoryDescription
                .expect("Category id was not null, but the category description was"),
            hex_color: record.CategoryHexColor
                .expect("Category id was not null, but the category hex color was"),
            amount: None,
        });
    }

    if let Some(id) = record.ExternalAccountId {
        transaction.external_account = Some(ExternalAccountDto {
            id,
            name: record.ExternalAccountEntityName
                .expect("External account id was not null, the the external account name was"),
            description: record.ExternalAccountDescription
                .expect("External account id was not null, the the external account description was"),
            default_category_id: record.ExternalAccounDefaultCategoryId,
        })
    }

    transaction
}
