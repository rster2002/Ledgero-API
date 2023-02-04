use rocket::Route;
use rocket::serde::json::Json;
use crate::error::http_error::HttpError;
use crate::models::dto::aggregates::user_total_dto::UserTotalDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_aggregate_routes() -> Vec<Route> {
    routes![
        get_user_total_amount,
    ]
}

#[get("/total")]
pub async fn get_user_total_amount(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<UserTotalDto>> {
    let pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT SUM(CompleteAmount)::bigint AS Total
            FROM Transactions
            WHERE TransactionType = 'transaction' AND UserId = $1
        "#,
        user.uuid
    )
        .fetch_one(pool)
        .await?;

    Ok(Json(UserTotalDto {
        total: record.total.ok_or::<Error>(
            HttpError::new(500)
                .message("Failed to calculate aggregate")
                .into()
        )?,
    }))
}
