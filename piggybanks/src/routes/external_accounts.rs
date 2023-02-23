use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::external_accounts::external_account_name_dto::ExternalAccountNameDto;
use crate::models::dto::external_accounts::new_external_account_dto::NewExternalAccountDto;
use crate::models::dto::external_accounts::new_external_account_name_dto::NewExternalAccountNameDto;
use crate::models::entities::category::Category;
use crate::models::entities::external_account::ExternalAccount;
use crate::models::entities::external_account_names::ExternalAccountName;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;
use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::queries::transactions_query::TransactionQuery;

pub fn create_external_account_routes() -> Vec<Route> {
    routes![
        get_all_external_accounts,
        create_new_external_account,
        get_external_account_by_id,
        update_external_account,
        delete_external_account,
        get_external_account_names,
        new_external_account_name,
        delete_external_account_name,
        get_transactions_for_external_account,
    ]
}

#[get("/")]
pub async fn get_all_external_accounts(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<ExternalAccountDto>>> {
    let pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT *
            FROM ExternalAccounts
            WHERE UserId = $1;
        "#,
        user.uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(
        records
            .into_iter()
            .map(|record| ExternalAccountDto {
                id: record.id,
                name: record.name,
                description: record.description,
                default_category_id: record.defaultcategoryid,
            })
            .collect(),
    ))
}

#[post("/", data = "<body>")]
pub async fn create_new_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewExternalAccountDto>,
) -> Result<Json<ExternalAccountDto>> {
    let pool = pool.inner();
    let body = body.0;

    if let Some(category_id) = &body.default_category_id {
        Category::guard_one(pool, category_id, &user.uuid).await?;
    }

    let external_account = ExternalAccount {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid,
        name: body.name,
        description: body.description,
        default_category_id: body.default_category_id,
    };

    external_account.create(pool).await?;

    Ok(Json(ExternalAccountDto {
        id: external_account.id,
        name: external_account.name,
        description: external_account.description,
        default_category_id: external_account.default_category_id,
    }))
}

#[get("/<id>")]
pub async fn get_external_account_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<ExternalAccountDto>> {
    let pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT *
            FROM ExternalAccounts
            WHERE Id = $1 AND UserId = $2
        "#,
        id,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    Ok(Json(ExternalAccountDto {
        id,
        name: record.name,
        description: record.description,
        default_category_id: record.defaultcategoryid,
    }))
}

#[put("/<id>", data = "<body>")]
pub async fn update_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewExternalAccountDto>,
) -> Result<Json<ExternalAccountDto>> {
    let pool = pool.inner();
    let body = body.0;

    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    if let Some(category_id) = &body.default_category_id {
        Category::guard_one(pool, category_id, &user.uuid).await?;
    }

    sqlx::query!(
        r#"
            UPDATE ExternalAccounts
            SET Name = $3, Description = $4, DefaultCategoryId = $5
            WHERE Id = $1 AND UserId = $2
        "#,
        id,
        user.uuid,
        body.name,
        body.description,
        body.default_category_id
    )
    .execute(pool)
    .await?;

    Ok(Json(ExternalAccountDto {
        id,
        name: body.name,
        description: body.description,
        default_category_id: body.default_category_id,
    }))
}

#[delete("/<id>")]
pub async fn delete_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let pool = pool.inner();

    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    sqlx::query!(
        r#"
            DELETE FROM ExternalAccounts
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[get("/<id>/names")]
pub async fn get_external_account_names(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<Vec<ExternalAccountNameDto>>> {
    let pool = pool.inner();

    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    let records = sqlx::query!(
        r#"
            SELECT *
            FROM ExternalAccountNames
            WHERE UserId = $1;
        "#,
        user.uuid
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(
        records
            .into_iter()
            .map(|record| ExternalAccountNameDto {
                id: record.id,
                name: record.name,
                parent_external_account: record.parentexternalaccount,
            })
            .collect(),
    ))
}

#[post("/<id>/names", data = "<body>")]
pub async fn new_external_account_name(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewExternalAccountNameDto>,
) -> Result<Json<ExternalAccountNameDto>> {
    let pool = pool.inner();
    let body = body.0;

    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    let external_account_name = ExternalAccountName {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid,
        name: body.name,
        parent_external_account: id,
    };

    external_account_name.create(pool).await?;

    Ok(Json(ExternalAccountNameDto {
        id: external_account_name.id,
        name: external_account_name.name,
        parent_external_account: external_account_name.parent_external_account,
    }))
}

#[delete("/<id>/names/<name_id>")]
pub async fn delete_external_account_name(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    name_id: String,
) -> Result<()> {
    let pool = pool.inner();

    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    ExternalAccountName::guard_one(pool, &name_id, &user.uuid).await?;

    sqlx::query!(
        r#"
            DELETE FROM ExternalAccountNames
            WHERE Id = $1 AND UserId = $2;
        "#,
        name_id,
        user.uuid
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[get("/<id>/transactions?<pagination..>")]
pub async fn get_transactions_for_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let inner_pool = pool.inner();

    let transactions = TransactionQuery::new(&user.uuid)
        .where_external_account(id)
        .order()
        .paginate(&pagination)
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(pagination, transactions)))
}
