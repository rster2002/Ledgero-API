use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use crate::models::entities::category::Category;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;
use rocket::serde::json::Json;
use sqlx::types::time::OffsetDateTime;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::entities::subcategory::Subcategory;
use crate::queries::transactions_query::{TransactionListQuery};

#[get("/?<pagination..>")]
pub async fn get_all_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let pool = pool.inner();

    let transactions = TransactionListQuery::new(&user.uuid)
        .where_type(TransactionType::Transaction)
        .order()
        .paginate(&pagination)
        .fetch_all(pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(pagination, transactions)))
}

#[get("/<id>")]
pub async fn get_single_transaction(
    id: String,
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<TransactionDto>> {
    let pool = pool.inner();

    let transaction = TransactionListQuery::new(&user.uuid)
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
                category_id,
                subcategory_id
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


