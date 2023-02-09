use rocket::Route;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::categories::new_category_dto::NewCategoryDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::category::Category;

use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::transactions::get_transactions::{map_record, TransactionRecord};
use crate::shared_types::SharedPool;

pub fn create_category_routes() -> Vec<Route> {
    routes![
        get_all_categories,
        create_new_category,
        get_category_by_id,
        update_category,
        delete_category,
        get_category_transactions,
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
    let inner_pool = pool.inner();

    let uuid = Uuid::new_v4();
    let category = Category {
        id: uuid.to_string(),
        user_id: user.uuid.to_string(),
        name: body.name,
        description: body.description,
        hex_color: body.hex_color,
    };

    category.create(inner_pool)
        .await?;

    get_category_by_id(pool, user, uuid.to_string())
        .await
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
    let inner_pool = pool.inner();

    Category::guard_one(inner_pool, &id, &user.uuid)
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
        .execute(inner_pool)
        .await?;

    get_category_by_id(pool, user, id)
        .await
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

#[get("/<id>/transactions")]
pub async fn get_category_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<Vec<TransactionDto>>> {
    let pool = pool.inner();

    Category::guard_one(pool, &id, &user.uuid)
        .await?;

    let records = sqlx::query_as!(
        TransactionRecord,
        r#"
            SELECT
                transactions.Id as TransactionId, TransactionType, FollowNumber, OriginalDescription, transactions.Description, CompleteAmount, Amount, ExternalAccountName,
                c.Id as "CategoryId?", c.Name as "CategoryName?", c.Description as "CategoryDescription?", c.HexColor as "CategoryHexColor?",
                b.Id as BankAccountId, b.Iban as BankAccountIban, b.Name as BankAccountName, b.Description as BankAccountDescription, b.HexColor as BankAccountHexColor,
                e.Id as "ExternalAccountId?", e.Name as "ExternalAccountEntityName?", e.Description as "ExternalAccountDescription?", e.DefaultCategoryId as "ExternalAccounDefaultCategoryId?"
            FROM Transactions
            LEFT JOIN categories c on transactions.categoryid = c.id
            LEFT JOIN bankaccounts b on transactions.bankaccountid = b.id
            LEFT JOIN externalaccounts e on c.id = e.defaultcategoryid
            WHERE CategoryId = $1 AND Transactions.UserId = $2;
        "#,
        id,
        user.uuid
    )
        .fetch_all(pool)
        .await?;

    let transactions = records
        .into_iter()
        .map(map_record)
        .collect();

    Ok(Json(transactions))
}
