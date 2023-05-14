use jumpdrive_auth::errors::TotpError;
use rocket::http::Status;
use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub struct WrapperTotpError {
    inner: TotpError,
}

impl WrapperTotpError {
    pub fn new(error: TotpError) -> Self {
        Self { inner: error }
    }
}

impl ToErrorDto for WrapperTotpError {
    fn get_status_code(&self) -> Status {
        match self.inner {
            TotpError::FailedToDecodeSecret => Status::InternalServerError,
            TotpError::InvalidOneTimePassword => Status::Unauthorized,
        }
    }

    fn get_description(&self) -> String {
        let slice = match self.inner {
            TotpError::FailedToDecodeSecret => "Internal server error",
            TotpError::InvalidOneTimePassword => "Invalid One Time Password",
        };

        slice.to_string()
    }
}
