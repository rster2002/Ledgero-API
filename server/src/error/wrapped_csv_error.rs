use csv::ErrorKind;
use rocket::http::Status;

use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub struct WrappedCsvError {
    inner: csv::Error,
}

impl WrappedCsvError {
    pub fn new(error: csv::Error) -> Self {
        Self { inner: error }
    }
}

impl ToErrorDto for WrappedCsvError {
    fn get_status_code(&self) -> Status {
        match self.inner.kind() {
            ErrorKind::Io(_) => Status::BadRequest,
            ErrorKind::Utf8 { .. } => Status::BadRequest,
            ErrorKind::UnequalLengths { .. } => Status::BadRequest,
            ErrorKind::Serialize(_) => Status::BadRequest,
            ErrorKind::Deserialize { .. } => Status::BadRequest,
            _ => Status::InternalServerError,
        }
    }

    fn get_description(&self) -> String {
        match self.inner.kind() {
            ErrorKind::Io(_) => "Failed reading the CSV content",
            ErrorKind::Utf8 { .. } => {
                "Failed to read the CSV content as it was not encoded in UTF-8"
            }
            ErrorKind::UnequalLengths { .. } => {
                "Failed to read the CSV content as not all rows have the number of columns"
            }
            _ => "Something went wrong on our end",
        }
        .to_string()
    }
}
