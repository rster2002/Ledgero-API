use rocket::Responder;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorDTO {
    pub error: ErrorContent,
}

#[derive(Debug, Serialize)]
pub struct ErrorContent {
    pub code: u16,
    pub reason: String,
    pub description: String,
}
