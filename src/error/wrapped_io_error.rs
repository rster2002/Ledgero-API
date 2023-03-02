use std::io;
use std::io::ErrorKind;
use rocket::http::Status;
use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub struct WrappedIoError {
    inner: io::Error,
}

impl WrappedIoError {
    pub fn new(error: io::Error) -> Self {
        Self {
            inner: error,
        }
    }
}

impl ToErrorDto for WrappedIoError {
    fn get_status_code(&self) -> Status {
        match self.inner.kind() {
            ErrorKind::NotFound => Status::NotFound,
            _ => Status::InternalServerError,
        }
    }

    fn get_description(&self) -> String {
        match self.inner.kind() {
            ErrorKind::NotFound => "Not found".to_string(),
            _ => "Internal Server Error".to_string(),
        }
    }
}
