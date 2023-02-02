use crate::models::dto::error_dto::ErrorDTO;
use rocket::http::Status;

#[derive(Debug)]
pub struct HttpError {
    code: u16,
    message: Option<String>,
}

impl HttpError {
    pub fn new(code: u16) -> Self {
        Self {
            code,
            message: None,
        }
    }

    pub fn from_status(status: Status) -> Self {
        HttpError::new(status.code)
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

impl HttpError {
    pub fn get_body(&self) -> Option<String> {
        let Some(message) = &self.message else {
            return None;
        };

        let json_string = serde_json::to_string(&ErrorDTO {
            message: message.to_string(),
        })
        .expect("Failed to serialize error DTO");

        Some(json_string)
    }
}
