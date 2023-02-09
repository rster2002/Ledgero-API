use crate::error::error_dto_trait::ToErrorDto;
use rocket::http::Status;
use std::fmt::{Debug, Display, Formatter};

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
    NotEnoughPermissions,
    MissingToken,
}

impl Display for JwtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JWT: {:?}", self)
    }
}

impl std::error::Error for JwtError {}

impl ToErrorDto for JwtError {
    fn get_status_code(&self) -> Status {
        match self {
            JwtError::PayloadIsNotJson
            | JwtError::PayloadNotAnObject
            | JwtError::NotAnAccessToken
            | JwtError::MissingHeader
            | JwtError::MissingPayload
            | JwtError::MissingSignature
            | JwtError::MissingToken => Status::Unauthorized,

            JwtError::InvalidSignature
            | JwtError::UsedBeforeNotBeforeClaim
            | JwtError::UsedAfterExpireClaim
            | JwtError::NotEnoughPermissions => Status::Forbidden,
        }
    }

    fn get_description(&self) -> String {
        let slice = match self {
            JwtError::PayloadIsNotJson => "Payload is not correct JSON",
            JwtError::PayloadNotAnObject => "Payload is not a JSON object",
            JwtError::NotAnAccessToken => "The given token is not an access token",
            JwtError::MissingHeader => "The JWT token doesn't have a header",
            JwtError::MissingPayload => "The JWT token doesn't have a payload",
            JwtError::MissingSignature => "The JWT token doesn't have a signature",
            JwtError::InvalidSignature => "The signature of the JWT payload is incorrect",
            JwtError::UsedBeforeNotBeforeClaim => "The token was used too early",
            JwtError::UsedAfterExpireClaim => "The token has expired",
            JwtError::NotEnoughPermissions => {
                "The current token does not have enough permissions to perform this action"
            }
            JwtError::MissingToken => "There is no token present in the request",
        };

        slice.to_string()
    }
}
