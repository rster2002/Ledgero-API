use std::fmt::{Display, Formatter};
use rocket::http::Status;
use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub enum AuthError {
    UsernameLessThanFourChars,
    PasswordLessThanEightChars,
    UsernameNotFound,
    PasswordIncorrect,
    NoUserForGrant,
    NoGrantWithId,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let slice = match self {
            AuthError::UsernameLessThanFourChars => "Username has to have at least four characters.",
            AuthError::PasswordLessThanEightChars => "Password has to have at least four characters.",
            AuthError::UsernameNotFound => "No user found with given username.",
            AuthError::PasswordIncorrect => "Provided password is incorrect.",
            AuthError::NoUserForGrant => "No user with the give id was found. The user might have been deleted.",
            AuthError::NoGrantWithId => "The given refresh token has been revoked.",
        };

        write!(f, "{}", slice)
    }
}

impl ToErrorDto for AuthError {
    fn get_status_code(&self) -> Status {
        match self {
            AuthError::UsernameLessThanFourChars
            | AuthError::PasswordLessThanEightChars => Status::BadRequest,

            AuthError::UsernameNotFound
            | AuthError::PasswordIncorrect => Status::Unauthorized,

            AuthError::NoUserForGrant => Status::Forbidden,
            AuthError::NoGrantWithId => Status::Unauthorized,
        }
    }

    fn get_description(&self) -> String {
        self.to_string()
    }
}
