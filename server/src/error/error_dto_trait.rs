use rocket::http::Status;

use crate::models::dto::error_dto::{ErrorContent, ErrorDTO};

pub trait ToErrorDto {
    fn get_status_code(&self) -> Status;
    fn get_description(&self) -> String;

    fn get_reason(&self) -> String {
        self.get_status_code().reason_lossy().to_string()
    }

    fn to_error_dto(&self) -> ErrorDTO {
        ErrorDTO {
            error: ErrorContent {
                code: self.get_status_code().code,
                reason: self.get_reason(),
                description: self.get_description(),
            },
        }
    }
}
