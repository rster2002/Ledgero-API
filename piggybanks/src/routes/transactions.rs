use rocket::http::Status;
use rocket::Route;
use rocket::serde::json::Json;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_transaction_routes() -> Vec<Route> {
    routes![
        get_all_transactions,
        get_single_transaction,
    ]
}

#[get("/")]
pub async fn get_all_transactions(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<TransactionDto>>> {
    todo!()
}

#[get("/<id>")]
pub async fn get_single_transaction(
    id: String,
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<TransactionDto>> {
    todo!()
}
