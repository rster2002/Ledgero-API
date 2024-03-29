use rocket::serde::json::Json;
use uuid::Uuid;

use crate::db_inner;
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
use crate::shared::SharedPool;

#[get("/<category_id>/subcategories/<subcategory_id>")]
pub async fn get_subcategory_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: &str,
    subcategory_id: &str,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(amount)::bigint
                FROM transactions
                WHERE subcategories.parent_category = transactions.category_id AND subcategories.id = transactions.subcategory_id
            )::bigint AS amount
            FROM subcategories
            WHERE id = $1 AND parent_category = $2 AND user_id = $3;
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
        hex_color: record.hex_color,
        amount: record.amount.unwrap_or(0),
    }))
}

#[delete("/<category_id>/subcategories/<subcategory_id>")]
pub async fn delete_subcategory(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: String,
    subcategory_id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    sqlx::query!(
        r#"
            DELETE FROM subcategories
            WHERE id = $1 AND parent_category = $2 AND user_id = $3;
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
    category_id: &str,
) -> Result<Json<Vec<SubcategoryDto>>> {
    let inner_pool = db_inner!(pool);

    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(amount)::bigint
                FROM transactions
                WHERE subcategories.parent_category = transactions.category_id AND subcategories.id = transactions.subcategory_id
            )::bigint AS amount
            FROM subcategories
            WHERE parent_category = $1 AND user_id = $2;
        "#,
        category_id,
        user.uuid
    )
        .fetch_all(inner_pool)
        .await?;

    let subcategories = records
        .into_iter()
        .map(|record| SubcategoryDto {
            id: record.id,
            name: record.name,
            description: record.description,
            hex_color: record.hex_color,
            amount: record.amount.unwrap_or(0),
        })
        .collect();

    Ok(Json(subcategories))
}

#[post("/<category_id>/subcategories", data = "<body>")]
pub async fn create_subcategory<'a>(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: &'a str,
    body: Json<NewSubcategoryDto<'a>>,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    Category::guard_one(inner_pool, category_id, &user.uuid).await?;

    let subcategory = Subcategory {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        parent_category: category_id.to_string(),
        name: body.name.to_string(),
        description: body.description.to_string(),
        hex_color: body.hex_color.to_string(),
    };

    subcategory.create(inner_pool).await?;

    get_subcategory_by_id(pool, user, &subcategory.parent_category, &subcategory.id).await
}

#[put("/<category_id>/subcategories/<subcategory_id>", data = "<body>")]
pub async fn update_subcategory<'a>(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: &'a str,
    subcategory_id: &'a str,
    body: Json<NewSubcategoryDto<'a>>,
) -> Result<Json<SubcategoryDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    sqlx::query!(
        r#"
            UPDATE subcategories
            SET name = $4, description = $5, hex_color = $6
            WHERE id = $1 AND parent_category = $2 AND user_id = $3;
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

    get_subcategory_by_id(pool, user, category_id, subcategory_id).await
}

#[get("/<category_id>/subcategories/<subcategory_id>/transactions?<pagination..>")]
pub async fn get_subcategory_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    category_id: &str,
    subcategory_id: &str,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let inner_pool = db_inner!(pool);

    let transactions = TransactionQuery::new(&user.uuid)
        .where_category(category_id)
        .where_subcategory(subcategory_id)
        .order()
        .paginate(&pagination)
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(
        pagination,
        transactions,
    )))
}
