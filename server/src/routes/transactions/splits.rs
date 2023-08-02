use rocket::serde::json::Json;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::transactions::new_split_dto::NewSplitDto;
use crate::models::dto::transactions::split_dto::SplitDto;
use crate::models::entities::category::Category;
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

    Transaction::guard_one(pool, transaction_id, &user.uuid).await?;

    let records = sqlx::query_as!(
        SplitRecord,
        r#"
            SELECT
                transactions.id, transactions.description, amount,
                c.id as "category_id?", c.name as "category_name?", c.description as "category_description?", c.hex_color as "category_hex_color?"
            FROM transactions
            LEFT JOIN categories c on transactions.category_id = c.id
            WHERE transaction_type = 'split' AND transactions.user_id = $1 AND parent_transaction_id = $2;
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

    if body.category_id.is_none() && body.subcategory_id.is_some() {
        return HttpError::new(400)
            .message("Cannot set a subcategory with providing a category id")
            .into();
    }

    // Check that the category and subcategory exists when they're not set to null.
    trace!("Checking category and subcategory exist");
    if let Some(category_id) = &body.category_id {
        Category::guard_one(inner_pool, category_id, &user.uuid).await?;

        if let Some(subcategory_id) = &body.subcategory_id {
            sqlx::query!(
                r#"
                    SELECT id
                    FROM subcategories
                    WHERE id = $1 AND parent_category = $2;
                "#,
                subcategory_id,
                category_id
            )
            .fetch_one(inner_pool)
            .await?;
        }
    }

    // Anything other than a 'transaction' type should should not be allowed to create a split
    sqlx::query!(
        r#"
            SELECT transaction_type
            FROM transactions
            WHERE id = $1 AND user_id = $2 AND transaction_type = 'transaction';
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
            SELECT amount, parent_transaction_id
            FROM transactions
            WHERE transaction_type = 'split' AND id = $1 AND user_id = $2;
        "#,
        split_id,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    let Some(parent_id) = split_record.parent_transaction_id else {
        return HttpError::new(404)
            .message("Could not find a split with the given id for this transaction")
            .into();
    };

    if parent_id != transaction_id {
        return HttpError::new(404)
            .message("Could not find a split with the given id for this transaction")
            .into();
    }

    let transaction_record = sqlx::query!(
        r#"
            SELECT amount
            FROM transactions
            WHERE transaction_type = 'transaction' AND id = $1 AND user_id = $2;
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
            UPDATE transactions
            SET amount = $3
            WHERE id = $1 AND user_id = $2;
        "#,
        transaction_id,
        user.uuid,
        new_transaction_amount
    )
    .execute(&mut *db_transaction)
    .await?;

    sqlx::query!(
        r#"
            DELETE FROM transactions
            WHERE id = $1 AND user_id = $2;
        "#,
        split_id,
        user.uuid
    )
    .execute(&mut *db_transaction)
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
