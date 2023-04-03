use rocket::serde::json::Json;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
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
use crate::routes::categories::subcategories::subcategory_by_id;
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
                    SELECT Id
                    FROM Subcategories
                    WHERE Id = $1 AND ParentCategory = $2;
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
            UPDATE Transactions
            SET CategoryId = $3, SubcategoryId = $4
            WHERE Id = $1 AND UserId = $2
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
            UPDATE Transactions
            SET Description = $3, CategoryId = $4, SubcategoryId = $5, ExternalAccountId = $6
            WHERE Id = $1 AND UserId = $2;
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
            subcategory_by_id(pool, user.clone(), category_id, subcategory_id).await?;
        }
    }

    trace!("Updating transactions in the database");
    sqlx::query!(
        r#"
            UPDATE Transactions
            SET Description = $3, CategoryId = $4, SubcategoryId = $5, ExternalAccountId = $6, Amount = CompleteAmount
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid,
        body.description,
        body.category_id,
        body.subcategory_id,
        body.external_account_id
    )
    .execute(&mut db_transaction)
    .await?;

    trace!("Deleting original splits");
    sqlx::query!(
        r#"
            DELETE FROM Transactions
            WHERE UserId = $1 AND ParentTransactionId = $2 AND TransactionType = 'split';
        "#,
        user.uuid,
        Some(id.to_string())
    )
    .execute(&mut db_transaction)
    .await?;

    trace!("Creating new splits");
    for split in body.splits {
        db_transaction = SplitService::create_split(db_transaction, &user.uuid, id, split).await?;
    }

    trace!("Committing database transaction");
    db_transaction.commit().await?;

    debug!("Updated entire transaction '{}'", id);
    get_single_transaction(pool, user.clone(), id).await
}
