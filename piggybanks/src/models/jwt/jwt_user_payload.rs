use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtUserPayload {
    pub uuid: String,
    pub username: String,
}
