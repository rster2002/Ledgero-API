pub mod error_dto_trait;
pub mod http_error;
pub mod import_error;
pub mod jwt_error;
pub mod wrapped_csv_error;
pub mod wrapped_sqlx_error;

use rocket::http::{ContentType, Status};
use rocket::response::Responder;

use crate::error::error_dto_trait::ToErrorDto;
use crate::error::http_error::HttpError;
use crate::error::import_error::ImportError;
use crate::error::jwt_error::JwtError;
use crate::error::wrapped_csv_error::WrappedCsvError;
use crate::error::wrapped_sqlx_error::WrappedSqlxError;
use crate::models::dto::error_dto::{ErrorContent, ErrorDTO};
use base64_url::base64::DecodeError;
use chrono::ParseError;
use rocket::{Request, Response};
use std::io;
use std::io::Cursor;
use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    IO(io::Error),
    DotEnv(dotenv::Error),
    Sqlx(WrappedSqlxError),
    Status(Status),
    SerdeJson(serde_json::Error),
    DecodeError(DecodeError),
    Utf8Error(FromUtf8Error),
    JwtError(JwtError),
    HttpError(HttpError),
    Csv(WrappedCsvError),
    ImportError(ImportError),
}

impl Error {
    pub fn generic(message: impl Into<String>) -> Error {
        Error::Generic(message.into())
    }
}

impl From<dotenv::Error> for Error {
    fn from(value: dotenv::Error) -> Self {
        Error::DotEnv(value)
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sqlx(WrappedSqlxError::new(value))
    }
}

impl From<Status> for Error {
    fn from(value: Status) -> Self {
        Error::Status(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

impl From<DecodeError> for Error {
    fn from(value: DecodeError) -> Self {
        Error::DecodeError(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Error::Utf8Error(value)
    }
}

impl From<JwtError> for Error {
    fn from(value: JwtError) -> Self {
        Error::JwtError(value)
    }
}

impl From<HttpError> for Error {
    fn from(value: HttpError) -> Self {
        Error::HttpError(value)
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::Csv(WrappedCsvError::new(value))
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Generic(value.into())
    }
}

impl From<ParseIntError> for Error {
    fn from(_value: ParseIntError) -> Self {
        Error::generic("Failed to parse int")
    }
}

impl From<ParseFloatError> for Error {
    fn from(_value: ParseFloatError) -> Self {
        Error::generic("Failed to parse int")
    }
}

impl From<chrono::ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::generic(format!("Failed parsing date time: {}", error))
    }
}

impl From<ImportError> for Error {
    fn from(value: ImportError) -> Self {
        Error::ImportError(value)
    }
}

impl Error {
    fn get_status_code(&self) -> u16 {
        match self {
            Error::JwtError(error) => error.get_status_code().code,
            Error::ImportError(error) => error.get_status_code(),
            Error::Sqlx(error) => error.get_status_code().code,
            Error::Csv(error) => error.get_status_code().code,
            Error::HttpError(error) => error.get_status_code().code,
            Error::SerdeJson(_) => Status::BadRequest.code,
            _ => 500,
        }
    }

    fn get_body(&self) -> String {
        let error_dto = match self {
            Error::Sqlx(error) => error.to_error_dto(),
            Error::Csv(error) => error.to_error_dto(),
            Error::HttpError(error) => error.to_error_dto(),
            Error::JwtError(error) => error.to_error_dto(),
            _ => ErrorDTO {
                error: ErrorContent {
                    code: 500,
                    reason: "Internal Server Error".to_string(),
                    description: "An unknown error occurred".to_string(),
                },
            },
        };

        serde_json::to_string(&error_dto).expect("Failed to serialize error dto")
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let body = self.get_body();

        Response::build()
            .header(ContentType::JSON)
            .status(Status::new(self.get_status_code()))
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
