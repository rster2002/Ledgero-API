use rocket::Route;
use rocket::serde::json::Json;
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_importing_routes() -> Vec<Route> {
    routes![
        import_csv,
    ]
}

#[post("/csv", data="<body>")]
pub async fn import_csv(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<ImportCsvDto>,
) -> Result<()> {
    todo!()
}
