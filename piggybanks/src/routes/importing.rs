pub mod csv_import;

use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use std::io::Cursor;

use rocket::serde::json::Json;
use rocket::Route;

use crate::error::import_error::ImportError;
use crate::models::csv::csv_mapping::{AmountMapping, DateMapping};
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use crate::models::entities::bank_account::BankAccount;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::entities::transaction::Transaction;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::{DbPool, SharedPool};
use uuid::Uuid;
use crate::models::dto::import::import_dto::ImportDto;
use crate::models::entities::import::Import;
use crate::routes::importing::csv_import::import_csv;

pub fn create_importing_routes() -> Vec<Route> {
    routes![
        import_csv,
        get_all_imports,
        get_import_by_id,
        delete_import,
    ]
}

#[get("/")]
pub async fn get_all_imports(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<ImportDto>>> {
    let inner_pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT *
            FROM Imports
            WHERE UserId = $1;
        "#,
        user.uuid
    )
        .fetch_all(inner_pool)
        .await?;

    let imports = records.into_iter()
        .map(|record| {
            ImportDto {
                id: record.id,
                imported_at: record.importedat.to_string(),
                filename: record.filename,
            }
        })
        .collect();

    Ok(Json(imports))
}

#[get("/<id>")]
pub async fn get_import_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<Vec<ImportDto>>> {
    todo!()
}

#[delete("/<id>")]
pub async fn delete_import(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<Vec<ImportDto>>> {
    todo!()
}

