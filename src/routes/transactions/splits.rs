use rocket::serde::json::Json;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::transactions::new_split_dto::NewSplitDto;
use crate::models::dto::transactions::split_dto::SplitDto;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::services::split_service::SplitService;
use crate::shared::SharedPool;

struct SplitRecord {
    pub id: String,
    pub description: String,
    pub amount: i64,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub category_description: Option<String>,
    pub category_hex_color: Option<String>,
}

#[get("/<transaction_id>/splits")]
pub async fn get_splits(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: &str,
) -> Result<Json<Vec<SplitDto>>> {
    let pool = db_inner!(pool);

    Transaction::guard_one(pool, &transaction_id, &user.uuid).await?;

    let records = sqlx::query_as!(
        SplitRecord,
        r#"
            SELECT
                transactions.Id, transactions.Description, Amount,
                c.Id as "category_id?", c.Name as "category_name?", c.Description as "category_description?", c.HexColor as "category_hex_color?"
            FROM Transactions
            LEFT JOIN categories c on transactions.categoryid = c.id
            WHERE TransactionType = 'split' AND transactions.UserId = $1 AND ParentTransactionId = $2;
        "#,
        user.uuid,
        transaction_id
    )
        .fetch_all(pool)
        .await?;

    Ok(Json(records.into_iter().map(map_split_record).collect()))
}

#[post("/<transaction_id>/splits", data = "<body>")]
pub async fn create_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: &str,
    body: Json<NewSplitDto<'_>>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    // Anything other than a 'transaction' type should should not be allowed to create a split
    sqlx::query!(
        r#"
            SELECT TransactionType
            FROM Transactions
            WHERE Id = $1 AND UserId = $2 AND TransactionType = 'transaction';
        "#,
        transaction_id,
        user.uuid
    )
        .fetch_one(inner_pool)
        .await?;

    let mut db_transaction = inner_pool.begin().await?;

    db_transaction =
        SplitService::create_split(db_transaction, &user.uuid, transaction_id, body.0).await?;

    db_transaction.commit().await?;

    debug!("Created split");
    Ok(())
}

#[put("/<transaction_id>/splits/<split_id>", data = "<body>")]
pub async fn update_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: &str,
    split_id: &str,
    body: Json<NewSplitDto<'_>>,
) -> Result<()> {
    let mut db_transaction = db_inner!(pool).begin().await?;

    db_transaction =
        SplitService::update_split(db_transaction, &user.uuid, transaction_id, split_id, body.0)
            .await?;

    db_transaction.commit().await?;

    debug!("Updated split");
    Ok(())
}

#[delete("/<transaction_id>/splits/<split_id>")]
pub async fn delete_split(
    pool: &SharedPool,
    user: JwtUserPayload,
    transaction_id: &str,
    split_id: &str,
) -> Result<()> {
    let pool = db_inner!(pool);

    let split_record = sqlx::query!(
        r#"
            SELECT Amount, ParentTransactionId
            FROM Transactions
            WHERE TransactionType = 'split' AND Id = $1 AND UserId = $2;
        "#,
        split_id,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    let Some(parent_id) = split_record.parenttransactionid else {
        return Err(
            HttpError::new(404)
                .message("Could not find a split with the given id for this transaction")
                .into()
        );
    };

    if parent_id != transaction_id {
        return Err(HttpError::new(404)
            .message("Could not find a split with the given id for this transaction")
            .into());
    }

    let transaction_record = sqlx::query!(
        r#"
            SELECT Amount
            FROM Transactions
            WHERE TransactionType = 'transaction' AND Id = $1 AND UserId = $2;
        "#,
        transaction_id,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    let new_transaction_amount = transaction_record.amount + split_record.amount;

    let mut db_transaction = pool.begin().await?;

    sqlx::query!(
        r#"
            UPDATE Transactions
            SET Amount = $3
            WHERE Id = $1 AND UserId = $2;
        "#,
        transaction_id,
        user.uuid,
        new_transaction_amount
    )
    .execute(&mut db_transaction)
    .await?;

    sqlx::query!(
        r#"
            DELETE FROM Transactions
            WHERE Id = $1 AND UserId = $2;
        "#,
        split_id,
        user.uuid
    )
    .execute(&mut db_transaction)
    .await?;

    db_transaction.commit().await?;

    debug!("Deleted split");
    Ok(())
}

fn map_split_record(record: SplitRecord) -> SplitDto {
    let mut split_dto = SplitDto {
        id: record.id,
        description: record.description,
        amount: record.amount,
        category: None,
    };

    if let Some(id) = record.category_id {
        split_dto.category = Some(SlimCategoryDto {
            id,
            name: record
                .category_name
                .expect("Category id was not null, but the category name was"),
            description: record
                .category_description
                .expect("Category id was not null, but the category description was"),
            hex_color: record
                .category_hex_color
                .expect("Category id was not null, but the category hex color was"),
        });
    }

    split_dto
}
