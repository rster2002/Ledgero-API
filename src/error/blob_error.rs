use rocket::http::Status;
use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub enum BlobError {
    NoMimeType,
}

impl ToErrorDto for BlobError {
    fn get_status_code(&self) -> Status {
        match self {
            BlobError::NoMimeType => Status::BadRequest,
        }
    }

    fn get_description(&self) -> String {
        match self {
            BlobError::NoMimeType => "The mime type could not be inferred".to_string()
        }
    }
}
