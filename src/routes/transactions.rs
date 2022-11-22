use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use crate::models::dto::transaction::CreateTransaction;

pub async fn create_transaction(Json(CreateTransaction): Json<CreateTransaction>) -> impl IntoResponse {
    (StatusCode::CREATED)
}
