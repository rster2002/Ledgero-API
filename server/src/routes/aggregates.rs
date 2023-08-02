use rocket::serde::json::Json;
use rocket::Route;

use crate::db_inner;
use crate::models::dto::aggregates::user_total_dto::UserTotalDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared::SharedPool;

pub fn create_aggregate_routes() -> Vec<Route> {
    routes![get_user_total_amount,]
}

#[get("/total")]
pub async fn get_user_total_amount(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<UserTotalDto>> {
    let pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT SUM(complete_amount)::bigint AS total
            FROM transactions
            WHERE transaction_type = 'transaction' AND user_id = $1
        "#,
        user.uuid
    )
    .fetch_one(pool)
    .await?;

    Ok(Json(UserTotalDto {
        total: record.total.unwrap_or(0),
    }))
}
