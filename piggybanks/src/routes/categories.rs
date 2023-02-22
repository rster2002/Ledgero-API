pub mod subcategories;

use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::categories::new_category_dto::NewCategoryDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::category::Category;
use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;

use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transaction_list_query::{TransactionListQuery, TransactionRecord};
use crate::routes::categories::subcategories::{create_subcategory, delete_subcategory, get_subcategories, subcategory_by_id, update_subcategory};
use crate::shared_types::SharedPool;

pub fn create_category_routes() -> Vec<Route> {
    routes![
        get_all_categories,
        create_new_category,
        get_category_by_id,
        update_category,
        delete_category,
        get_category_transactions,
        subcategory_by_id,
        delete_subcategory,
        get_subcategories,
        create_subcategory,
        update_subcategory,
    ]
}

#[get("/")]
pub async fn get_all_categories(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<CategoryDto>>> {
    let pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)::bigint
                FROM Transactions
                WHERE Categories.Id = Transactions.CategoryId
            ) AS Amount
            FROM Categories
            WHERE UserId = $1;
        "#,
        user.uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(
        records
            .into_iter()
            .map(|record| CategoryDto {
                id: record.id,
                name: record.name,
                description: record.description,
                hex_color: record.hexcolor,
                amount: record.amount
                    .unwrap_or(0),
                subcategories: vec![],
            })
            .collect(),
    ))
}

#[post("/", data = "<body>")]
pub async fn create_new_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewCategoryDto>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;
    let inner_pool = pool.inner();

    let uuid = Uuid::new_v4();
    let category = Category {
        id: uuid.to_string(),
        user_id: user.uuid.to_string(),
        name: body.name,
        description: body.description,
        hex_color: body.hex_color,
    };

    category.create(inner_pool).await?;

    get_category_by_id(pool, user, uuid.to_string()).await
}

#[get("/<id>")]
pub async fn get_category_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<CategoryDto>> {
    let pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)::bigint
                FROM Transactions
                WHERE Categories.Id = Transactions.CategoryId
            ) AS Amount
            FROM Categories
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    Ok(Json(CategoryDto {
        id: record.id,
        name: record.name,
        description: record.description,
        hex_color: record.hexcolor,
        amount: record.amount
            .unwrap_or(0),
        subcategories: vec![],
    }))
}

#[put("/<id>", data = "<body>")]
pub async fn update_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewCategoryDto>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;
    let inner_pool = pool.inner();

    Category::guard_one(inner_pool, &id, &user.uuid).await?;

    sqlx::query!(
        r#"
            UPDATE Categories
            SET Name = $3, Description = $4, HexColor = $5
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid,
        body.name,
        body.description,
        body.hex_color,
    )
    .execute(inner_pool)
    .await?;

    get_category_by_id(pool, user, id).await
}

#[delete("/<id>")]
pub async fn delete_category(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    let pool = pool.inner();

    Category::guard_one(pool, &id, &user.uuid).await?;

    sqlx::query!(
        r#"
            DELETE FROM Categories
            WHERE Id = $1 AND UserId = $2
        "#,
        id,
        user.uuid
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[get("/<id>/transactions?<pagination..>")]
pub async fn get_category_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let pool = pool.inner();

    Category::guard_one(pool, &id, &user.uuid).await?;

    let transactions = TransactionListQuery::new(&user.uuid)
        .where_category(&id)
        .order()
        .paginate(&pagination)
        .fetch_all(pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(pagination, transactions)))
}
