use rocket::serde::json::Json;
use rocket::Route;

use crate::db_inner;
use crate::models::dto::import::import_dto::ImportDto;
use crate::models::dto::import::import_dto_with_numbers::ImportDtoWithNumbers;
use crate::models::entities::import::Import;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::importing::check_csv_mapping::check_csv_mapping as check_csv_mapping_route;
use crate::routes::importing::csv_import::import_csv;
use crate::shared::SharedPool;

pub mod check_csv_mapping;
pub mod csv_import;
pub mod map_csv_record;

pub fn create_importing_routes() -> Vec<Route> {
    routes![
        import_csv,
        check_csv_mapping_route,
        get_all_imports,
        get_import_by_id,
        delete_import,
    ]
}

#[get("/")]
pub async fn get_all_imports(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<ImportDtoWithNumbers>>> {
    let inner_pool = db_inner!(pool);

    let records = sqlx::query!(
        r#"
            SELECT *, (
                SELECT COUNT(id)
                FROM transactions
                WHERE parent_import = imports.id
            )::int AS imported,
            (
                SELECT COUNT(follow_number)
                FROM skipped_transactions
                WHERE import_id = imports.id
            )::int AS skipped
            FROM imports
            WHERE user_id = $1
            ORDER BY imported_at DESC;
        "#,
        user.uuid
    )
    .fetch_all(inner_pool)
    .await?;

    let imports = records
        .into_iter()
        .map(|record| ImportDtoWithNumbers {
            id: record.id,
            imported_at: record.imported_at.to_string(),
            filename: record.file_name,
            imported: record.imported.expect("Expected a number"),
            skipped: record.skipped.expect("Expected a number"),
        })
        .collect();

    Ok(Json(imports))
}

#[get("/<id>")]
pub async fn get_import_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<ImportDto>> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT *
            FROM imports
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    Ok(Json(ImportDto {
        id: record.id,
        imported_at: record.imported_at.to_string(),
        filename: record.file_name,
    }))
}

#[delete("/<id>")]
pub async fn delete_import(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    let inner_pool = db_inner!(pool);

    Import::guard_one(inner_pool, &id, &user.uuid).await?;

    sqlx::query!(
        r#"
            DELETE FROM imports
            WHERE id = $1 AND user_id = $2;
        "#,
        id,
        user.uuid
    )
    .execute(inner_pool)
    .await?;

    debug!("Deleted import '{}'", id);
    Ok(())
}
