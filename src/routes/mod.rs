mod transactions;

use axum::{
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    Json, Router,
};

pub fn create_router() -> Router {
    Router::new()
        .route()
}
