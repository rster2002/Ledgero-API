use rocket::Route;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::categories::new_category_dto::NewCategoryDto;
use crate::models::entities::category::Category;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_category_routes() -> Vec<Route> {
    routes![
        get_all_categories,
        create_new_category,
        get_category_by_id,
        update_category,
        delete_category,
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
        records.into_iter()
            .map(|record| {
                CategoryDto {
                    id: record.id,
                    name: record.name,
                    description: record.description,
                    hex_color: record.hexcolor,
                    amount: record.amount,
                }
            })
            .collect()
    ))
}

#[post("/", data="<body>")]
pub async fn create_new_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewCategoryDto>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;

    let category = Category {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        name: body.name,
        description: body.description,
        hex_color: body.hex_color,
    };

    category.create(pool)
        .await?;

    Ok(Json(CategoryDto {
        id: category.id,
        name: category.name,
        description: category.description,
        hex_color: category.hex_color,
        amount: Some(0),
    }))
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
        amount: record.amount,
    }))
}

#[put("/<id>", data="<body>")]
pub async fn update_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewCategoryDto>,
) -> Result<Json<CategoryDto>> {
    let body = body.0;
    let pool = pool.inner();

    Category::guard_one(pool, &id, &user.uuid)
        .await?;

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
        .execute(pool)
        .await?;

    Ok(Json(CategoryDto {
        id,
        name: body.name,
        description: body.description,
        hex_color: body.hex_color,
        amount: None,
    }))
}

#[delete("/<id>")]
pub async fn delete_category(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let pool = pool.inner();

    Category::guard_one(pool, &id, &user.uuid)
        .await?;

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
