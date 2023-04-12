

use rocket::http::Status;
use jumpdrive_auth::errors::JwtError;

use crate::error::error_dto_trait::ToErrorDto;

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

            _ => Status::InternalServerError,
        }
    }

    fn get_description(&self) -> String {
        self.to_string()
    }
}
