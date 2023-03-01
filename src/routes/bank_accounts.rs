use rocket::Route;
use rocket::serde::json::Json;
use sqlx::Error::Database;
use crate::error::http_error::HttpError;
use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::bank_accounts::update_bank_account_dto::UpdateBankAccountDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_bank_account_routes() -> Vec<Route> {
    routes![
        get_bank_accounts,
        get_bank_account_by_id,
        update_bank_account,
        delete_bank_account,
    ]
}

#[get("/")]
pub async fn get_bank_accounts(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<BankAccountDto>>> {
    let inner_pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT *
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
    let inner_pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT *
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
    }))
}

#[put("/<id>", data="<body>")]
pub async fn update_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<UpdateBankAccountDto>,
) -> Result<Json<BankAccountDto>> {
    let inner_pool = pool.inner();
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

    get_bank_account_by_id(pool, user, id)
        .await
}

#[delete("/<id>")]
pub async fn delete_bank_account(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<()> {
    let inner_pool = pool.inner();

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

    Ok(())
}
