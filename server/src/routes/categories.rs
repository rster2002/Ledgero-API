use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;

use crate::db_inner;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::categories::new_category_dto::NewCategoryDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::category::Category;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::categories_query::CategoriesQuery;
use crate::queries::transactions_query::TransactionQuery;
use crate::routes::categories::moving::{delete_move, move_money_between_categories};
use crate::routes::categories::ordering::category_ordering;
use crate::routes::categories::subcategories::{
    create_subcategory, delete_subcategory, get_subcategories, get_subcategory_transactions,
    get_subcategory_by_id, update_subcategory,
};
use crate::shared::SharedPool;

pub mod ordering;
pub mod subcategories;
pub mod moving;

pub fn create_category_routes() -> Vec<Route> {
    routes![
        get_all_categories,
        create_new_category,
        get_category_by_id,
        update_category,
        delete_category,
        get_category_transactions,
        get_subcategory_by_id,
        delete_subcategory,
        get_subcategories,
        create_subcategory,
        update_subcategory,
        get_subcategory_transactions,
        category_ordering,
        move_money_between_categories,
        delete_move,
    ]
}

#[get("/")]
pub async fn get_all_categories(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<CategoryDto>>> {
    let pool = db_inner!(pool);

    debug!("Querying all categories for user '{}'", user);
    let categories = CategoriesQuery::new(&user.uuid)
        .order()
        .fetch_all(pool)
        .await?;

    Ok(Json(categories))
}

#[post("/", data = "<body>")]
pub async fn create_new_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewCategoryDto<'_>>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;
    let inner_pool = db_inner!(pool);

    trace!("Finding max order index");
    let ordering_index = sqlx::query!(
        r#"
            SELECT MAX(order_index) AS max_index
            FROM categories
            WHERE user_id = $1;
        "#,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    let category = Category {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        name: body.name.to_string(),
        description: body.description.to_string(),
        hex_color: body.hex_color.to_string(),
        ordering_index: ordering_index.max_index.unwrap_or(0) + 1,
    };

    debug!("Creating new category for user '{}'", user);
    category.create(inner_pool).await?;

    debug!("Created category '{}'", category.id);
    get_category_by_id(pool, user, &category.id).await
}

#[get("/<id>")]
pub async fn get_category_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &str,
) -> Result<Json<CategoryDto>> {
    let pool = db_inner!(pool);

    debug!("Querying category with id '{}'", id);
    let category = CategoriesQuery::new(&user.uuid)
        .where_id(id)
        .fetch_one(pool)
        .await?;

    Ok(Json(category))
}

#[put("/<id>", data = "<body>")]
pub async fn update_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: &str,
    body: Json<NewCategoryDto<'_>>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;
    let inner_pool = db_inner!(pool);

    debug!("Executing category guard for id '{}' with user '{}'", id, user);
    Category::guard_one(inner_pool, id, &user.uuid).await?;

    debug!("Updating category with id '{}'", id);
    sqlx::query!(
        r#"
            UPDATE categories
            SET name = $3, description = $4, hex_color = $5
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid,
        body.name,
        body.description,
        body.hex_color,
    )
    .execute(inner_pool)
    .await?;

    debug!("Updated category '{}'", id);
    get_category_by_id(pool, user, id).await
}

#[delete("/<id>")]
pub async fn delete_category(pool: &SharedPool, user: JwtUserPayload, id: &str) -> Result<()> {
    let pool = db_inner!(pool);

    debug!("Executing category guard for id '{}' with user '{}'", id, user);
    Category::guard_one(pool, id, &user.uuid).await?;

    debug!("Deleting category with id '{}'", id);
    sqlx::query!(
        r#"
            DELETE FROM categories
            WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.uuid
    )
    .execute(pool)
    .await?;

    debug!("Deleted category '{}'", id);
    Ok(())
}

#[get("/<id>/transactions?<pagination..>")]
pub async fn get_category_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let pool = db_inner!(pool);

    debug!("Executing category guard for id '{}' with user '{}'", id, user);
    Category::guard_one(pool, &id, &user.uuid).await?;

    let transactions = TransactionQuery::new(&user.uuid)
        .where_category(&id)
        .order()
        .paginate(&pagination)
        .fetch_all(pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(
        pagination,
        transactions,
    )))
}
