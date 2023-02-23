use rocket::serde::json::Json;
use uuid::Uuid;
use crate::models::dto::categories::subcategories::new_subcategory_dto::NewSubcategoryDto;
use crate::models::dto::categories::subcategories::subcategory_dto::SubcategoryDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::category::Category;
use crate::models::entities::subcategory::Subcategory;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transactions_query::TransactionQuery;
use crate::routes::categories::get_category_by_id;
use crate::shared_types::SharedPool;

#[get("/<category_id>/subcategories/<subcategory_id>")]
pub async fn subcategory_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    subcategory_id: String,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)::bigint
                FROM Transactions
                WHERE Subcategories.ParentCategory = Transactions.CategoryId AND Subcategories.Id = Transactions.SubcategoryId
            )::bigint AS Amount
            FROM Subcategories
            WHERE Id = $1 AND ParentCategory = $2 AND UserId = $3;
        "#,
        subcategory_id,
        category_id,
        user.uuid
    )
        .fetch_one(inner_pool)
        .await?;

    Ok(Json(SubcategoryDto {
        id: record.id,
        name: record.name,
        description: record.description,
        hex_color: record.hexcolor,
        amount: record.amount
            .unwrap_or(0),
    }))
}

#[delete("/<category_id>/subcategories/<subcategory_id>")]
pub async fn delete_subcategory(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    subcategory_id: String,
) -> Result<()> {
    let inner_pool = pool.inner();

    sqlx::query!(
        r#"
            DELETE FROM Subcategories
            WHERE Id = $1 AND ParentCategory = $2 AND UserId = $3;
        "#,
        subcategory_id,
        category_id,
        user.uuid
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}

#[get("/<category_id>/subcategories")]
pub async fn get_subcategories(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
) -> Result<Json<Vec<SubcategoryDto>>> {
    let inner_pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)::bigint
                FROM Transactions
                WHERE Subcategories.ParentCategory = Transactions.CategoryId AND Subcategories.Id = Transactions.SubcategoryId
            )::bigint AS Amount
            FROM Subcategories
            WHERE ParentCategory = $1 AND UserId = $2;
        "#,
        category_id,
        user.uuid
    )
        .fetch_all(inner_pool)
        .await?;

    let subcategories = records.into_iter()
        .map(|record| {
            SubcategoryDto {
                id: record.id,
                name: record.name,
                description: record.description,
                hex_color: record.hexcolor,
                amount: record.amount
                    .unwrap_or(0),
            }
        })
        .collect();

    Ok(Json(subcategories))
}

#[post("/<category_id>/subcategories", data="<body>")]
pub async fn create_subcategory(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    body: Json<NewSubcategoryDto>,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = pool.inner();
    let body = body.0;

    Category::guard_one(pool, &category_id, &user.uuid)
        .await?;

    let subcategory = Subcategory {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        parent_category: category_id,
        name: body.name,
        description: body.description,
        hex_color: body.hex_color,
    };

    subcategory.create(pool)
        .await?;

    subcategory_by_id(pool, user, subcategory.parent_category, subcategory.id)
        .await
}

#[put("/<category_id>/subcategories/<subcategory_id>", data="<body>")]
pub async fn update_subcategory(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    subcategory_id: String,
    body: Json<NewSubcategoryDto>,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = pool.inner();
    let body = body.0;

    sqlx::query!(
        r#"
            UPDATE Subcategories
            SET Name = $4, Description = $5, HexColor = $6
            WHERE Id = $1 AND ParentCategory = $2 AND UserId = $3;
        "#,
        subcategory_id,
        category_id,
        user.uuid,
        body.name,
        body.description,
        body.hex_color
    )
        .execute(inner_pool)
        .await?;

    subcategory_by_id(pool, user, category_id, subcategory_id)
        .await
}

#[get("/<category_id>/subcategories/<subcategory_id>/transactions?<pagination..>")]
pub async fn get_subcategory_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    subcategory_id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let inner_pool = pool.inner();

    let transactions = TransactionQuery::new(&user.uuid)
        .where_category(category_id)
        .where_subcategory(subcategory_id)
        .order()
        .paginate(&pagination)
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(pagination, transactions)))
}
