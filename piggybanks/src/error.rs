pub mod jwt_error;

use rocket::http::{ContentType, Status};
use rocket::response::{Responder};

use rocket::{Request, Response};
use std::io;
use std::io::Cursor;
use std::string::FromUtf8Error;
use base64_url::base64::DecodeError;
use crate::error::jwt_error::JwtError;

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

impl Error {
    fn get_status_code(&self) -> u16 {
        match self {
            Error::JwtError(error) => error.get_status_code(),
            Error::SerdeJson(_) => Status::BadRequest.code,
            _ => 500
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'o> {
        let err = format!("{:?}", self);

        Response::build()
            .header(ContentType::JSON)
            .sized_body(err.len(), Cursor::new(err))
            .status(Status::new(self.get_status_code()))
            .ok()
    }
}
