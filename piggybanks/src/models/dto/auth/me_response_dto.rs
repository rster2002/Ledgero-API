use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub uuid: String,
    pub username: String,
}
