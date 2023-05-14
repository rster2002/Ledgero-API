use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableMfaDto {
    pub secret_key: String,
    pub code: String,
}
