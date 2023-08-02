use rocket::serde::json::Json;
use rocket::Route;
use sqlx::Error::Database;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::bank_accounts::update_bank_account_dto::UpdateBankAccountDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::pagination::pagination_response_dto::PaginationResponseDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::queries::transactions_query::TransactionQuery;
use crate::shared::SharedPool;

pub fn create_bank_account_routes() -> Vec<Route> {
    routes![
        get_bank_accounts,
        get_bank_account_by_id,
        update_bank_account,
        delete_bank_account,
        get_transactions_for_bank_account,
    ]
}

#[get("/")]
pub async fn get_bank_accounts(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<BankAccountDto>>> {
    let inner_pool = db_inner!(pool);

    debug!("Querying all bank accounts for user '{}'", user);
    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(amount)
                FROM transactions
                WHERE transactions.bank_account_id = bank_accounts.id
            )::bigint AS amount
            FROM bank_accounts
            WHERE user_id = $1;
        "#,
        user.uuid
    )
    .fetch_all(inner_pool)
    .await?;

    trace!("Mapping bank account records");
    let bank_accounts = records
        .into_iter()
        .map(|record| BankAccountDto {
            id: record.id,
            iban: record.iban,
            name: record.name,
            description: record.description,
            hex_color: record.hex_color,
            amount: record.amount.unwrap_or(0),
        })
        .collect();

    Ok(Json(bank_accounts))
}

#[get("/<id>")]
pub async fn get_bank_account_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<BankAccountDto>> {
    let inner_pool = db_inner!(pool);

    debug!("Querying database for bank account with id '{}'", id);
    let record = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(amount)
                FROM transactions
                WHERE transactions.bank_account_id = bank_accounts.id
            )::bigint AS amount
            FROM bank_accounts
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    trace!("Returning bank account entry");
    Ok(Json(BankAccountDto {
        id: record.id,
        iban: record.iban,
        name: record.name,
        description: record.description,
        hex_color: record.hex_color,
        amount: record.amount.unwrap_or(0),
    }))
}

#[put("/<id>", data = "<body>")]
pub async fn update_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<UpdateBankAccountDto>,
) -> Result<Json<BankAccountDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    debug!("Updating bank account '{}'", id);
    sqlx::query!(
        r#"
            UPDATE bank_accounts
            SET name = $3, description = $4, hex_color = $5
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid,
        body.name,
        body.description,
        body.hex_color
    )
    .execute(inner_pool)
    .await?;

    debug!("Updated bank account '{}'", id);
    get_bank_account_by_id(pool, user, id).await
}

#[delete("/<id>")]
pub async fn delete_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    debug!("Deleting bank account '{}'", id);
    let result = sqlx::query!(
        r#"
            DELETE FROM bank_accounts
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid
    )
    .execute(inner_pool)
    .await;

    if let Err(error) = result {
        let Database(db_error) = &error else {
            return Err(error.into());
        };

        let Some(constraint) = db_error.constraint() else {
            return Err(error.into())
        };

        if constraint != "transactions_bank_account_id_fkey" {
            return Err(error.into());
        }

        trace!("Failed to delete bank account due to transaction constraints");
        return Err(
            HttpError::new(409) // Conflict
                .message(
                    "Cannot delete a bank account that still has transactions associated with it",
                )
                .into(),
        );
    }

    debug!("Deleted bank account '{}'", id);
    Ok(())
}

#[get("/<id>/transactions?<pagination..>")]
pub async fn get_transactions_for_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    pagination: PaginationQueryDto,
) -> Result<Json<PaginationResponseDto<TransactionDto>>> {
    let inner_pool = db_inner!(pool);

    let transactions = TransactionQuery::new(user.uuid)
        .where_type_not(TransactionType::Split)
        .where_bank_account(id)
        .order()
        .paginate(&pagination)
        .fetch_all(inner_pool)
        .await?;

    Ok(Json(PaginationResponseDto::from_query(
        pagination,
        transactions,
    )))
}
