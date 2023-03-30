use rocket::http::Status;
use crate::error::Error;
use crate::prelude::*;

use crate::error::error_dto_trait::ToErrorDto;

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

impl ToErrorDto for HttpError {
    fn get_status_code(&self) -> Status {
        Status::new(self.code)
    }

    fn get_description(&self) -> String {
        let Some(message) = &self.message else {
            return "Unknown error".to_string();
        };

        message.to_string()
    }
}

impl<T> Into<Result<T>> for HttpError {
    fn into(self) -> Result<T> {
        Err(Error::HttpError(self))
    }
}
