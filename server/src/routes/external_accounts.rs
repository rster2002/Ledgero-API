use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;

use crate::db_inner;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::external_accounts::external_account_name_dto::ExternalAccountNameDto;
use crate::models::dto::external_accounts::new_external_account_dto::NewExternalAccountDto;
use crate::models::dto::external_accounts::new_external_account_name_dto::NewExternalAccountNameDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::category::Category;
use crate::models::entities::external_account::ExternalAccount;
use crate::models::entities::external_account_names::ExternalAccountName;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transactions_query::TransactionQuery;
use crate::shared::SharedPool;

pub fn create_external_account_routes() -> Vec<Route> {
    routes![
        get_all_external_accounts,
        create_new_external_account,
        get_external_account_by_id,
        update_external_account,
        delete_external_account,
        get_external_account_names,
        add_external_account_name,
        delete_external_account_name,
        get_transactions_for_external_account,
        apply_external_account_name,
        remove_external_account_name_associations,
    ]
}

#[get("/")]
pub async fn get_all_external_accounts(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<ExternalAccountDto>>> {
    let pool = db_inner!(pool);

    let records = sqlx::query!(
        r#"
            SELECT *
            FROM external_accounts
            WHERE user_id = $1;
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
                hex_color: record.hex_color,
                default_category_id: record.default_category_id,
                default_subcategory_id: record.default_subcategory_id,
            })
            .collect(),
    ))
}

#[post("/", data = "<body>")]
pub async fn create_new_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewExternalAccountDto<'_>>,
) -> Result<Json<ExternalAccountDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    if let Some(category_id) = &body.default_category_id {
        Category::guard_one(inner_pool, category_id, &user.uuid).await?;
    }

    let external_account = ExternalAccount {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid.to_string(),
        name: body.name.to_string(),
        description: body.description.to_string(),
        hex_color: body.hex_color.to_string(),
        default_category_id: body.default_category_id.map(|v| v.to_string()),
        default_subcategory_id: body.default_subcategory_id.map(|v| v.to_string()),
    };

    external_account.create(inner_pool).await?;

    debug!("Created external account '{}'", external_account.id);
    get_external_account_by_id(pool, user.clone(), external_account.id).await
}

#[get("/<id>")]
pub async fn get_external_account_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<ExternalAccountDto>> {
    let pool = db_inner!(pool);

    debug!("Querying external account '{}'", id);
    let record = sqlx::query!(
        r#"
            SELECT *
            FROM external_accounts
            WHERE id = $1 AND user_id = $2
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
        hex_color: record.hex_color,
        default_category_id: record.default_category_id,
        default_subcategory_id: record.default_subcategory_id,
    }))
}

#[put("/<id>", data = "<body>")]
pub async fn update_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewExternalAccountDto<'_>>,
) -> Result<Json<ExternalAccountDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    debug!("Executing external account guard for '{}' with user '{}'", id, user);
    ExternalAccount::guard_one(inner_pool, &id, &user.uuid).await?;

    if let Some(category_id) = &body.default_category_id {
        debug!("Executing category guard for '{}' with user '{}'", category_id, user);
        Category::guard_one(inner_pool, category_id, &user.uuid).await?;
    }

    debug!("Updating external account '{}'", id);
    sqlx::query!(
        r#"
            UPDATE external_accounts
            SET name = $3, description = $4, default_category_id = $5, default_subcategory_id = $6, hex_color = $7
            WHERE id = $1 AND user_id = $2
        "#,
        id,
        user.uuid,
        body.name,
        body.description,
        body.default_category_id,
        body.default_subcategory_id,
        body.hex_color
    )
    .execute(inner_pool)
    .await?;

    debug!("Updated external account '{}'", id);
    get_external_account_by_id(pool, user.clone(), id).await
}

#[delete("/<id>")]
pub async fn delete_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    debug!("Executing external account guard for '{}' with user '{}'", id, user);
    ExternalAccount::guard_one(inner_pool, &id, &user.uuid).await?;

    trace!("Deleting external account '{}'", id);
    sqlx::query!(
        r#"
            DELETE FROM external_accounts
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid
    )
    .execute(inner_pool)
    .await?;

    debug!("Deleted external account '{}'", id);
    Ok(())
}

#[get("/<id>/names")]
pub async fn get_external_account_names(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<Vec<ExternalAccountNameDto>>> {
    let inner_pool = db_inner!(pool);

    debug!("Executing external account guard for '{}' with user '{}'", id, user);
    ExternalAccount::guard_one(inner_pool, &id, &user.uuid).await?;

    trace!("Querying external account names");
    let records = sqlx::query!(
        r#"
            SELECT *
            FROM external_account_names
            WHERE user_id = $1 AND parent_external_account = $2;
        "#,
        user.uuid,
        id,
    )
    .fetch_all(inner_pool)
    .await?;

    trace!("Mapping external account names");
    Ok(Json(
        records
            .into_iter()
            .map(|record| ExternalAccountNameDto {
                id: record.id,
                name: record.name,
                parent_external_account: record.parent_external_account,
            })
            .collect(),
    ))
}

#[post("/<id>/names", data = "<body>")]
pub async fn add_external_account_name(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<NewExternalAccountNameDto<'_>>,
) -> Result<Json<ExternalAccountNameDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    debug!("Executing external account guard for '{}' with user '{}'", id, user);
    ExternalAccount::guard_one(inner_pool, &id, &user.uuid).await?;

    let external_account_name = ExternalAccountName {
        id: Uuid::new_v4().to_string(),
        user_id: user.uuid,
        name: body.name.to_string(),
        parent_external_account: id,
    };

    debug!("Creating new external account name '{}'", external_account_name.id);
    external_account_name.create(inner_pool).await?;

    debug!(
        "Added new external account name '{}' to '{}'",
        external_account_name.id, external_account_name.parent_external_account
    );
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
    let pool = db_inner!(pool);

    debug!("Executing external account guard for '{}' with user '{}'", id, user);
    ExternalAccount::guard_one(pool, &id, &user.uuid).await?;

    debug!("Executing external account name guard for '{}' with user '{}'", id, user);
    ExternalAccountName::guard_one(pool, &name_id, &user.uuid).await?;

    debug!("Deleting name '{}' from '{}'", name_id, id);
    sqlx::query!(
        r#"
            DELETE FROM external_account_names
            WHERE id = $1 AND user_id = $2;
        "#,
        name_id,
        user.uuid
    )
    .execute(pool)
    .await?;

    debug!("Deleted name '{}' from '{}'", name_id, id);
    Ok(())
}

#[get("/<id>/transactions?<pagination..>")]
pub async fn get_transactions_for_external_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let inner_pool = db_inner!(pool);

    let transactions = TransactionQuery::new(&user.uuid)
        .where_external_account(id)
        .order()
        .paginate(&pagination)
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(
        pagination,
        transactions,
    )))
}

#[patch("/<id>/names/<name_id>/apply")]
pub async fn apply_external_account_name(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    name_id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT name
            FROM external_account_names
            WHERE user_id = $1 AND parent_external_account = $2 AND id = $3;
        "#,
        user.uuid,
        id,
        name_id
    )
        .fetch_one(inner_pool)
        .await?;

    sqlx::query!(
        r#"
            UPDATE transactions
            SET external_account_id = $3, external_account_name_id = $4
            WHERE user_id = $1 AND external_account_name = $2;
        "#,
        user.uuid,
        record.name,
        id,
        name_id
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}

#[patch("/<id>/names/<name_id>/remove-associations")]
pub async fn remove_external_account_name_associations(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    name_id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    // Makes sure the external account exists
    sqlx::query!(
        r#"
            SELECT name
            FROM external_account_names
            WHERE user_id = $1 AND parent_external_account = $2 AND id = $3;
        "#,
        user.uuid,
        id,
        name_id
    )
        .fetch_one(inner_pool)
        .await?;

    sqlx::query!(
        r#"
            UPDATE transactions
            SET external_account_id = null, external_account_name_id = null
            WHERE user_id = $1 AND external_account_id = $2 AND external_account_name_id = $3;
        "#,
        user.uuid,
        id,
        name_id,
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}
