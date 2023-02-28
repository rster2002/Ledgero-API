use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use crate::models::entities::category::Category;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;
use rocket::serde::json::Json;

use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::update_transaction_dto::UpdateTransactionDto;

use crate::queries::transactions_query::TransactionQuery;

use crate::services::split_service::SplitService;

#[get("/?<pagination..>")]
pub async fn get_all_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let pool = pool.inner();

    let transactions = TransactionQuery::new(&user.uuid)
        .where_type(TransactionType::Transaction)
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
    id: String,
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<TransactionDto>> {
    let pool = pool.inner();

    let transaction = TransactionQuery::new(&user.uuid)
        .where_type(TransactionType::Transaction)
        .where_id(&id)
        .fetch_one(pool)
        .await?;

    Ok(Json(transaction))
}

#[patch("/<id>/category", data = "<body>")]
pub async fn change_category_for_transaction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<TransactionSetCategoryDto>,
) -> Result<()> {
    let pool = pool.inner();
    let body = body.0;

    Transaction::guard_one(pool, &id, &user.uuid).await?;

    if let Some(category_id) = &body.category_id {
        Category::guard_one(pool, category_id, &user.uuid).await?;

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
            .fetch_one(pool)
            .await?;
        }
    }

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
    .execute(pool)
    .await?;

    Ok(())
}

#[put("/<id>", data = "<body>")]
pub async fn update_transaction(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<UpdateTransactionDto<'_>>,
) -> Result<Json<TransactionDto>> {
    let inner_pool = pool.inner();
    let body = body.0;

    let _current_transaction = get_single_transaction(id.to_string(), pool, user.clone())
        .await?
        .0;

    let mut db_transaction = inner_pool.begin().await?;

    sqlx::query!(
        r#"
            UPDATE Transactions
            SET Description = $3, CategoryId = $4, SubcategoryId = $5, Amount = CompleteAmount
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid,
        body.description,
        body.category_id,
        body.subcategory_id
    )
    .execute(&mut db_transaction)
    .await?;

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

    for split in body.splits {
        db_transaction = SplitService::create_split(
            db_transaction,
            user.uuid.to_string(),
            id.to_string(),
            split,
        )
        .await?;
    }

    db_transaction.commit().await?;

    get_single_transaction(id.to_string(), pool, user.clone()).await
}
