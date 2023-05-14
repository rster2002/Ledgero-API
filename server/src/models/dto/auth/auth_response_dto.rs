pub mod jwt_access_token_payload;

use serde::Serialize;
use crate::models::dto::auth::auth_response_dto::jwt_access_token_payload::JwtAccessTokenPayload;

#[derive(Debug, Serialize)]
#[serde(tag = "response_type")]
pub enum AuthResponseDto {
    JwtAccessToken(JwtAccessTokenPayload),
    TwoFAChallenge,
}

#[cfg(test)]
impl AuthResponseDto {
    pub fn unwrap_jwt_access_token(self) -> JwtAccessTokenPayload {
        match self {
            AuthResponseDto::JwtAccessToken(payload) => payload,
            _ => panic!("Incorrect variant"),
        }
    }
}
