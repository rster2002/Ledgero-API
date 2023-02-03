pub mod http_error;
pub mod jwt_error;

use rocket::http::{ContentType, Status};
use rocket::response::Responder;

use crate::error::http_error::HttpError;
use crate::error::jwt_error::JwtError;
use base64_url::base64::DecodeError;
use rocket::{Request, Response};
use std::io;
use std::io::Cursor;
use std::num::{ParseFloatError, ParseIntError};
use std::string::FromUtf8Error;
use chrono::ParseError;
use crate::models::dto::error_dto::ErrorDTO;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    IO(io::Error),
    DotEnv(dotenv::Error),
    SQLX(sqlx::Error),
    Status(Status),
    SerdeJson(serde_json::Error),
    DecodeError(DecodeError),
    Utf8Error(FromUtf8Error),
    JwtError(JwtError),
    HttpError(HttpError),
    CSV(csv::Error)
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
        dbg!(&value);
        Error::SQLX(value)
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
        Error::CSV(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Generic(value.into())
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::generic("Failed to parse int")
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::generic("Failed to parse int")
    }
}

impl From<chrono::ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::generic(format!("Failed parsing date time: {}", error.to_string()))
    }
}

impl Error {
    fn get_status_code(&self) -> u16 {
        match self {
            Error::JwtError(error) => error.get_status_code(),
            Error::SerdeJson(_) => Status::BadRequest.code,
            _ => 500,
        }
    }

    fn get_body(&self) -> Option<String> {
        match self {
            Error::Generic(message) => {
                Some(
                    serde_json::to_string(&ErrorDTO {
                        message: message.to_string(),
                    })
                        .expect("Failed to serialize error dto")
                )
            },
            Error::IO(_) => Some("IO".to_string()),
            Error::DotEnv(_) => Some("DotEnv".to_string()),
            Error::SQLX(_) => Some("SQLX".to_string()),
            Error::Status(_) => Some("Status".to_string()),
            Error::SerdeJson(_) => Some("SerdeJson".to_string()),
            Error::DecodeError(_) => Some("DecodeError".to_string()),
            Error::Utf8Error(_) => Some("Utf8Error".to_string()),
            Error::JwtError(_) => Some("JwtError".to_string()),
            Error::HttpError(_) => Some("HttpError".to_string()),
            Error::CSV(_) => Some("Generic".to_string()),
            _ => None,
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let mut builder = Response::build();

        builder
            .header(ContentType::JSON)
            .status(Status::new(self.get_status_code()));

        if let Some(body) = self.get_body() {
            builder.sized_body(body.len(), Cursor::new(body));
        }

        builder.ok()
    }
}
