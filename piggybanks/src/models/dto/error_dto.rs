use rocket::Responder;
use serde::Serialize;

#[derive(Debug, Responder, Serialize)]
#[response(status = 500, content_type = "json")]
pub struct ErrorDTO {
    pub message: String,
}
