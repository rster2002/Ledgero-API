use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomMfaSecretKeyDto {
    pub secret_key: String,
}
