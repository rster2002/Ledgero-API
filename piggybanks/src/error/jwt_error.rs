use std::fmt::{Debug, Display, Formatter};
use rocket::http::Status;

#[derive(Debug)]
pub enum JwtError {
    PayloadIsNotJson,
    PayloadNotAnObject,
    NotAnAccessToken,
    MissingHeader,
    MissingPayload,
    MissingSignature,
    InvalidSignature,
    UsedBeforeNotBeforeClaim,
    UsedAfterExpireClaim,
}

impl JwtError {
    pub fn get_status_code(&self) -> u16 {
        let status = match self {
            JwtError::PayloadIsNotJson
            | JwtError::PayloadNotAnObject
            | JwtError::NotAnAccessToken
            | JwtError::MissingHeader
            | JwtError::MissingPayload
            | JwtError::MissingSignature => Status::Unauthorized,

            JwtError::InvalidSignature
            | JwtError::UsedBeforeNotBeforeClaim
            | JwtError::UsedAfterExpireClaim => Status::Forbidden,
        };

        status.code
    }
}

impl Display for JwtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JWT: {:?}", self)
    }
}

impl std::error::Error for JwtError {}
