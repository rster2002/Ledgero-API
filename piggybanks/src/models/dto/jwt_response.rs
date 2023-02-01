use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct JwtResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires: u32,
}
