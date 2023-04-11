use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RevokeDto {
    pub refresh_token: String,
}
