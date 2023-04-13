use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all="camelCase")]
pub struct RegistrationEnabledDto {
    pub enabled: bool,
}
