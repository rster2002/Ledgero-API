use chrono::Utc;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::error::http_error::HttpError;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::transactions::new_split_dto::NewSplitDto;
use crate::models::dto::transactions::split_dto::SplitDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

struct SplitRecord {
    pub id: String,
    pub description: String,
    pub amount: i64,
    pub CategoryId: Option<String>,
    pub CategoryName: Option<String>,
    pub CategoryDescription: Option<String>,
    pub CategoryHexColor: Option<String>,
}

#[get("/<transaction_id>/splits")]
pub async fn get_splits(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: String,
) -> Result<Json<Vec<SplitDto>>> {
    let pool = pool.inner();

    Transaction::guard_one(pool, &transaction_id, &user.uuid)
        .await?;

    let records = sqlx::query_as!(
        SplitRecord,
        r#"
            SELECT
                transactions.Id, transactions.Description, Amount,
                c.Id as "CategoryId?", c.Name as "CategoryName?", c.Description as "CategoryDescription?", c.HexColor as "CategoryHexColor?"
            FROM Transactions
            LEFT JOIN categories c on transactions.categoryid = c.id
            WHERE TransactionType = 'split' AND transactions.UserId = $1 AND ParentTransactionId = $2;
        "#,
        user.uuid,
        transaction_id
    )
        .fetch_all(pool)
        .await?;

    Ok(Json(
        records.into_iter()
            .map(|record| {
                map_split_record(record)
            })
            .collect()
    ))
}

#[post("/<transaction_id>/splits", data="<body>")]
pub async fn create_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: String,
    body: Json<NewSplitDto>
) -> Result<()> {
    let pool = pool.inner();
    let body = body.0;

    let parent_transaction = sqlx::query!(
        r#"
            SELECT Id, BankAccountId, Amount, ExternalAccountName, ExternalAccountId
            FROM Transactions
            WHERE Id = $1 AND UserId = $2;
        "#,
        transaction_id,
        user.uuid
    )
        .fetch_one(pool)
        .await?;

    if parent_transaction.amount > 0 && body.amount > parent_transaction.amount {
        return Err(
            HttpError::new(400)
                .message("Cannot create a split with an amount bigger than the remaining about of the parent")
                .into()
        );
    } else if parent_transaction.amount < 0 && body.amount < parent_transaction.amount {
        return Err(
            HttpError::new(400)
                .message("Cannot create a split with an amount bigger than the remaining about of the parent")
                .into()
        );
    }

    let db_transaction = pool.begin()
        .await?;

    let split_transaction = Transaction {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        transaction_type: TransactionType::Split,
        follow_number: Uuid::new_v4().to_string(),
        original_description: body.description.to_string(),
        description: body.description,
        complete_amount: body.amount,
        amount: body.amount,
        date: Utc::now().to_rfc3339(),
        bank_account_id: parent_transaction.bankaccountid,
        category_id: body.category_id,
        parent_transaction_id: Some(parent_transaction.id),
        external_account_name: parent_transaction.externalaccountname,
        external_account_id: parent_transaction.externalaccountid,
    };

    split_transaction.create(pool)
        .await?;

    let new_amount = parent_transaction.amount - split_transaction.amount;

    sqlx::query!(
        r#"
            UPDATE Transactions
            SET Amount = $3
            WHERE Id = $1 AND UserId = $2;
        "#,
        transaction_id,
        user.uuid,
        new_amount
    )
        .execute(pool)
        .await?;

    db_transaction.commit()
        .await?;

    Ok(())
}

#[put("/<transaction_id>/splits/<split_id>")]
pub async fn update_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: String,
    split_id: String,
) -> Result<Json<SplitDto>> {
    todo!()
}

#[delete("/<transaction_id>/splits/<split_id>")]
pub async fn delete_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: String,
    split_id: String,
) -> Result<Json<SplitDto>> {
    todo!()
}

fn map_split_record(record: SplitRecord) -> SplitDto {
    let mut split_dto = SplitDto {
        description: record.description,
        amount: record.amount,
        category: None,
    };

    if let Some(id) = record.CategoryId {
        split_dto.category = Some(CategoryDto {
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

    split_dto
}
