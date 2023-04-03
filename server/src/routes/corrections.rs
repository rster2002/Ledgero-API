use chrono::Utc;
use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::transactions::new_correction_dto::NewCorrectionDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transactions_query::TransactionQuery;

use crate::shared::SharedPool;

pub fn create_correction_routes() -> Vec<Route> {
    routes![
        get_all_corrections,
        create_correction,
        update_correction,
        delete_correction,
    ]
}

#[get("/")]
pub async fn get_all_corrections(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<TransactionDto>>> {
    let inner_pool = db_inner!(pool);

    let transactions = TransactionQuery::new(user.uuid)
        .where_type(TransactionType::Correction)
        .order()
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(transactions))
}

/// Creates a correction. A correction is not a real transaction, but instead is a transaction
/// that allows the user to correct their total for example when not all transactions are imported
/// in the tool or there is a difference between the actual total and the total in the tool.
#[post("/", data = "<body>")]
pub async fn create_correction(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewCorrectionDto>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    let record = sqlx::query!(
        r#"
            SELECT MAX(OrderIndicator)
            FROM Transactions
            WHERE UserId = $1;
        "#,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    let uuid = Uuid::new_v4();
    let _cloned_user = user.clone();

    let transaction = Transaction {
        id: uuid.to_string(),
        user_id: user.uuid.to_string(),
        transaction_type: TransactionType::Correction,
        follow_number: Uuid::new_v4().to_string(),
        original_description: body.description.to_string(),
        description: body.description.to_string(),
        complete_amount: body.amount,
        amount: body.amount,
        date: Utc::now(),
        bank_account_id: body.bank_account_id,
        category_id: body.category_id,
        parent_transaction_id: None,
        external_account_name: "Correction".to_string(),
        external_account_id: None,
        parent_import_id: None,
        subcategory_id: body.subcategory_id,
        order_indicator: record.max.unwrap_or(0) + 1,
    };

    transaction.create(inner_pool).await?;

    let transaction = TransactionQuery::new(user.uuid)
        .where_id(uuid.to_string())
        .fetch_one(inner_pool)
        .await?;

    debug!("Created correction '{}'", transaction.id);
    Ok(Json(transaction))
}

#[put("/<id>", data = "<body>")]
pub async fn update_correction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewCorrectionDto>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    // Checks if the transaction exists and if a correction
    sqlx::query!(
        r#"
            SELECT Id
            FROM Transactions
            WHERE Id = $1 AND UserId = $2 AND TransactionType = 'correction';
        "#,
        id,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    sqlx::query!(
        r#"
            UPDATE Transactions
            SET Amount = $3, CompleteAmount = $3, Description = $4, BankAccountId = $5, CategoryId = $6, SubcategoryId = $7
            WHERE Id = $1 AND UserId = $2 AND TransactionType = 'correction';
        "#,
        id,
        user.uuid,
        body.amount,
        body.description,
        body.bank_account_id,
        body.category_id,
        body.subcategory_id
    )
        .execute(inner_pool)
        .await?;

    let transaction = TransactionQuery::new(user.uuid)
        .where_id(id)
        .fetch_one(inner_pool)
        .await?;

    Ok(Json(transaction))
}

/// Usually transaction can only be deleted by deleting it's associated import, but corrections can
/// be deleted on their own.
#[delete("/<id>")]
pub async fn delete_correction(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT TransactionType
            FROM Transactions
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    let transaction_type = TransactionType::from(&*record.transactiontype);

    if transaction_type != TransactionType::Correction {
        return Err(HttpError::new(400) // Bad request
            .message("Cannot delete a transaction that is not a correction")
            .into());
    }

    sqlx::query!(
        r#"
            DELETE FROM Transactions
            WHERE Id = $1 AND UserId = $2 AND TransactionType = 'correction';
        "#,
        id,
        user.uuid
    )
    .execute(inner_pool)
    .await?;

    Ok(())
}
