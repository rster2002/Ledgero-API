use chrono::Utc;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::categories::move_between_categories_dto::MoveBetweenCategoriesDto;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared::SharedPool;

#[patch("/move", data="<body>")]
pub async fn move_money_between_categories(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<MoveBetweenCategoriesDto>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    let uuid_a = Uuid::new_v4();
    let uuid_b = Uuid::new_v4();

    let transaction_from = Transaction {
        id: uuid_a.to_string(),
        user_id: user.uuid.to_string(),
        transaction_type: TransactionType::Move,
        follow_number: Uuid::new_v4().to_string(),
        original_description: "".to_string(),
        description: "".to_string(),
        complete_amount: -(body.amount as i64),
        amount: -(body.amount as i64),
        date: Utc::now(),
        bank_account_id: None,
        category_id: Some(body.from_category_id),
        parent_transaction_id: None,
        external_account_name: "".to_string(),
        external_account_id: None,
        external_account_name_id: None,
        parent_import_id: None,
        subcategory_id: body.from_subcategory_id,
        order_indicator: 0,
        related_move_transaction: Some(uuid_b.to_string()),
    };

    let transaction_to = Transaction {
        id: uuid_b.to_string(),
        user_id: user.uuid.to_string(),
        transaction_type: TransactionType::Move,
        follow_number: Uuid::new_v4().to_string(),
        original_description: "".to_string(),
        description: "".to_string(),
        complete_amount: body.amount as i64,
        amount: body.amount as i64,
        date: Utc::now(),
        bank_account_id: None,
        category_id: Some(body.to_category_id),
        parent_transaction_id: None,
        external_account_name: "".to_string(),
        external_account_id: None,
        external_account_name_id: None,
        parent_import_id: None,
        subcategory_id: body.to_subcategory_id,
        order_indicator: 0,
        related_move_transaction: Some(uuid_a.to_string()),
    };

    let mut db_transaction = inner_pool.begin().await?;

    transaction_to.create(&mut *db_transaction).await?;
    transaction_from.create(&mut *db_transaction).await?;

    db_transaction.commit().await?;

    Ok(())
}

#[delete("/move/<id>")]
pub async fn delete_move(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    sqlx::query!(
        r#"
            SELECT transaction_type
            FROM transactions
            WHERE user_id = $1 AND transaction_type = 'move' AND Id = $2;
        "#,
        user.uuid,
        id
    )
        .fetch_one(inner_pool)
        .await?;

    sqlx::query!(
        r#"
            DELETE FROM transactions
            WHERE user_id = $1 AND transaction_type = 'move' AND id = $2;
        "#,
        user.uuid,
        id,
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}
