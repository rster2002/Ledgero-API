use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;
use base64_url::base64::DecodeError;

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
    FailedToSerializeJson,
    FailedToDecodeBase64Url,
    FailedToReadStringAsUtf8,
}

impl Display for JwtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JWT: {:?}", self)
    }
}

impl std::error::Error for JwtError {}

impl From<serde_json::Error> for JwtError {
    fn from(_: serde_json::Error) -> Self {
        JwtError::FailedToSerializeJson
    }
}

impl From<DecodeError> for JwtError {
    fn from(_: DecodeError) -> Self {
        JwtError::FailedToDecodeBase64Url
    }
}

impl From<FromUtf8Error> for JwtError {
    fn from(value: FromUtf8Error) -> Self {
        JwtError::FailedToReadStringAsUtf8
    }
}
