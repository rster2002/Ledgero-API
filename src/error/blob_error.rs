use rocket::http::Status;

use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub enum BlobError {
    NoMimeType,
    NoBlobToConfirm,
}

impl ToErrorDto for BlobError {
    fn get_status_code(&self) -> Status {
        match self {
            BlobError::NoMimeType => Status::BadRequest,
            BlobError::NoBlobToConfirm => Status::Conflict,
        }
    }

    fn get_description(&self) -> String {
        match self {
            BlobError::NoMimeType => "The mime type could not be inferred".to_string(),
            BlobError::NoBlobToConfirm => "No blob was found with the given token. It's either already confirmed or expired".to_string(),
        }
    }
}
