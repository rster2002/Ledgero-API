use rocket::Route;
use rocket::serde::json::Json;
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

    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)
                FROM Transactions
                WHERE Transactions.BankAccountId = BankAccounts.id
            )::bigint AS Amount
            FROM BankAccounts
            WHERE UserId = $1;
        "#,
        user.uuid
    )
        .fetch_all(inner_pool)
        .await?;

    let bank_accounts = records.into_iter()
        .map(|record| {
            BankAccountDto {
                id: record.id,
                iban: record.iban,
                name: record.name,
                description: record.description,
                hex_color: record.hexcolor,
                amount: record.amount
                    .unwrap_or(0),
            }
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

    let record = sqlx::query!(
        r#"
            SELECT *, (
                SELECT SUM(Amount)
                FROM Transactions
                WHERE Transactions.BankAccountId = BankAccounts.id
            )::bigint AS Amount
            FROM BankAccounts
            WHERE Id = $1 AND UserId = $2;
        "#,
        id,
        user.uuid
    )
        .fetch_one(inner_pool)
        .await?;

    Ok(Json(BankAccountDto {
        id: record.id,
        iban: record.iban,
        name: record.name,
        description: record.description,
        hex_color: record.hexcolor,
        amount: record.amount
            .unwrap_or(0),
    }))
}

#[put("/<id>", data="<body>")]
pub async fn update_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<UpdateBankAccountDto>,
) -> Result<Json<BankAccountDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    sqlx::query!(
        r#"
            UPDATE BankAccounts
            SET Name = $3, Description = $4, HexColor = $5
            WHERE Id = $1 AND UserId = $2;
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
    get_bank_account_by_id(pool, user, id)
        .await
}

#[delete("/<id>")]
pub async fn delete_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let result = sqlx::query!(
        r#"
            DELETE FROM BankAccounts
            WHERE Id = $1 AND UserId = $2;
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

        if constraint != "transactions_bankaccountid_fkey" {
            return Err(error.into())
        }

        return Err(
            HttpError::new(409) // Conflict
                .message("Cannot delete a bank account that still has transactions associated with it")
                .into()
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
