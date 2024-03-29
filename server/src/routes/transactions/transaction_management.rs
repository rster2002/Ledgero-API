use rocket::serde::json::Json;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::bulk_update_transaction_categories_dto::BulkUpdateTransactionCategoriesDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use crate::models::dto::transactions::update_transaction_details_dto::UpdateTransactionDetailsDto;
use crate::models::dto::transactions::update_transaction_dto::UpdateTransactionDto;
use crate::models::entities::category::Category;
use crate::models::entities::transaction::transaction_type::TransactionType;

use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transactions_query::TransactionQuery;
use crate::routes::categories::get_category_by_id;
use crate::routes::categories::subcategories::get_subcategory_by_id;
use crate::services::split_service::SplitService;
use crate::shared::SharedPool;

#[get("/?<pagination..>")]
pub async fn get_all_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let pool = db_inner!(pool);

    let transactions = TransactionQuery::new(&user.uuid)
        .where_type_not(TransactionType::Split)
        .order()
        .paginate(&pagination)
        .fetch_all(pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(
        pagination,
        transactions,
    )))
}

#[get("/<id>")]
pub async fn get_single_transaction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &str,
) -> Result<Json<TransactionDto>> {
    let pool = db_inner!(pool);

    let transaction = TransactionQuery::new(&user.uuid)
        .where_type(TransactionType::Transaction)
        .where_id(id)
        .fetch_one(pool)
        .await?;

    Ok(Json(transaction))
}

/// Only changes the category and subcategory of the given transaction. This endpoint works for all
/// types of transactions, like real- or correction transactions.
#[patch("/<id>/category", data = "<body>")]
pub async fn change_category_for_transaction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &str,
    body: Json<TransactionSetCategoryDto<'_>>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    get_single_transaction(pool, user.clone(), id).await?;

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

    trace!("Updating transaction");
    sqlx::query!(
        r#"
            UPDATE transactions
            SET category_id = $3, subcategory_id = $4
            WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.uuid,
        body.category_id,
        body.subcategory_id
    )
    .execute(inner_pool)
    .await?;

    debug!("Updated category for transaction '{}'", id);
    get_single_transaction(pool, user, id).await
}

#[patch("/<id>/details", data = "<body>")]
pub async fn update_transaction_details<'a>(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &'a str,
    body: Json<UpdateTransactionDetailsDto<'a>>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    // This also checks if the transaction has transactionType = 'transaction'
    get_single_transaction(pool, user.clone(), id).await?;

    trace!("Updating transactions in the database");
    sqlx::query!(
        r#"
            UPDATE transactions
            SET description = $3, category_id = $4, subcategory_id = $5, external_account_id = $6
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid,
        body.description,
        body.category_id,
        body.subcategory_id,
        body.external_account_id
    )
    .execute(inner_pool)
    .await?;

    debug!("Updated details for transaction '{}'", id);
    get_single_transaction(pool, user.clone(), id).await
}

#[put("/<id>", data = "<body>")]
pub async fn update_transaction<'a>(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &'a str,
    body: Json<UpdateTransactionDto<'a>>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    // This also checks if the transaction has transactionType = 'transaction'
    get_single_transaction(pool, user.clone(), id).await?;

    trace!("Starting database transaction");
    let mut db_transaction = inner_pool.begin().await?;

    if body.category_id.is_none() && body.subcategory_id.is_some() {
        return HttpError::new(400)
            .message("Cannot set a subcategory without defining a category")
            .into();
    }

    if let Some(category_id) = &body.category_id {
        get_category_by_id(pool, user.clone(), category_id).await?;

        if let Some(subcategory_id) = &body.subcategory_id {
            get_subcategory_by_id(pool, user.clone(), category_id, subcategory_id).await?;
        }
    }

    trace!("Updating transactions in the database");
    sqlx::query!(
        r#"
            UPDATE transactions
            SET description = $3, category_id = $4, subcategory_id = $5, external_account_id = $6, amount = complete_amount
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid,
        body.description,
        body.category_id,
        body.subcategory_id,
        body.external_account_id
    )
    .execute(&mut *db_transaction)
    .await?;

    trace!("Deleting original splits");
    sqlx::query!(
        r#"
            DELETE FROM transactions
            WHERE user_id = $1 AND parent_transaction_id = $2 AND transaction_type = 'split';
        "#,
        user.uuid,
        Some(id.to_string())
    )
    .execute(&mut *db_transaction)
    .await?;

    trace!("Creating new splits");
    for split in body.splits {
        SplitService::create_split(&mut db_transaction, &user.uuid, id, split)
            .await?;
    }

    trace!("Committing database transaction");
    db_transaction.commit().await?;

    debug!("Updated entire transaction '{}'", id);
    get_single_transaction(pool, user.clone(), id).await
}

#[patch("/bulk-update-categories", data="<body>")]
pub async fn bulk_update_transaction_categories(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<BulkUpdateTransactionCategoriesDto>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    if body.transactions.is_empty() {
        return HttpError::new(400)
            .message("List of transaction ids cannot be empty")
            .into();
    }

    if body.category_id.is_none() && body.subcategory_id.is_some() {
        return HttpError::new(404)
            .message("Cannot update subcategory without specifying a category")
            .into();
    }

    let record = sqlx::query!(
        r#"
            SELECT COUNT(*)
            FROM transactions
            WHERE user_id = $1 AND id = ANY($2);
        "#,
        user.uuid,
        &body.transactions[..]
    )
        .fetch_one(inner_pool)
        .await?;

    if record.count.unwrap_or(0) as usize != body.transactions.len() {
        return HttpError::new(404)
            .message("Not all transactions exist")
            .into();
    }

    sqlx::query!(
        r#"
            UPDATE transactions
            SET category_id = $3, subcategory_id = $4
            WHERE user_id = $1 AND id = ANY($2);
        "#,
        user.uuid,
        &body.transactions[..],
        body.category_id,
        body.subcategory_id
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}
