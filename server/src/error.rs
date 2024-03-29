use std::io;
use std::io::Cursor;
use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;

use base64_url::base64::DecodeError;
use chrono::ParseError;
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::time::error::ComponentRange;
use rocket::{Request, Response};
use jumpdrive_auth::errors::{JwtError, TotpError};
use memcache::MemcacheError;

use crate::error::blob_error::BlobError;
use crate::error::error_dto_trait::ToErrorDto;
use crate::error::http_error::HttpError;
use crate::error::import_error::ImportError;
use crate::error::wrapped_csv_error::WrappedCsvError;
use crate::error::wrapped_io_error::WrappedIoError;
use crate::error::wrapped_memcached_error::WrappedMemcachedError;
use crate::error::wrapped_sqlx_error::WrappedSqlxError;
use crate::error::wrapper_totp_error::WrapperTotpError;
use crate::models::dto::error_dto::{ErrorContent, ErrorDTO};

pub mod blob_error;
pub mod error_dto_trait;
pub mod http_error;
pub mod import_error;
pub mod jwt_error;
pub mod wrapped_csv_error;
pub mod wrapped_io_error;
pub mod wrapped_sqlx_error;
pub mod wrapper_totp_error;
pub mod wrapped_memcached_error;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    IO(WrappedIoError),
    Sqlx(WrappedSqlxError),
    SerdeJson(serde_json::Error),
    DecodeError(DecodeError),
    Utf8Error(FromUtf8Error),
    JwtError(JwtError),
    HttpError(HttpError),
    Csv(WrappedCsvError),
    ImportError(ImportError),
    BlobError(BlobError),
    TotpError(WrapperTotpError),
    MemcachedError(WrappedMemcachedError),
    RateLimitError,
}

impl Error {
    pub fn generic(message: impl Into<String>) -> Error {
        Error::Generic(message.into())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IO(WrappedIoError::new(value))
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::Sqlx(WrappedSqlxError::new(value))
    }
}

impl From<Status> for Error {
    fn from(value: Status) -> Self {
        Error::HttpError(HttpError::new(value.code).message(value.to_string()))
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

impl From<ComponentRange> for Error {
    fn from(_: ComponentRange) -> Self {
        Error::generic("ComponentRange")
    }
}

impl From<BlobError> for Error {
    fn from(value: BlobError) -> Self {
        Error::BlobError(value)
    }
}

impl From<TotpError> for Error {
    fn from(value: TotpError) -> Self {
        Error::TotpError(WrapperTotpError::new(value))
    }
}

impl From<MemcacheError> for Error {
    fn from(value: MemcacheError) -> Self {
        Error::MemcachedError(WrappedMemcachedError::new(value))
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
            Error::BlobError(error) => error.get_status_code().code,
            Error::IO(error) => error.get_status_code().code,
            Error::TotpError(error) => error.get_status_code().code,
            Error::MemcachedError(error) => error.get_status_code().code,
            Error::SerdeJson(_) => Status::BadRequest.code,
            Error::RateLimitError => Status::TooManyRequests.code,
            _ => 500,
        }
    }

    fn get_body(&self) -> String {
        let error_dto = match self {
            Error::Sqlx(error) => error.to_error_dto(),
            Error::Csv(error) => error.to_error_dto(),
            Error::HttpError(error) => error.to_error_dto(),
            Error::JwtError(error) => error.to_error_dto(),
            Error::BlobError(error) => error.to_error_dto(),
            Error::IO(error) => error.to_error_dto(),
            Error::TotpError(error) => error.to_error_dto(),
            Error::MemcachedError(error) => error.to_error_dto(),
            Error::RateLimitError => ErrorDTO {
                error: ErrorContent {
                    code: Status::TooManyRequests.code,
                    reason: "Too Many Requests".to_string(),
                    description: "Too many requests where send".to_string(),
                }
            },
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
