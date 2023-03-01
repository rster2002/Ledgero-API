use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExternalAccountNameDto {
    pub name: String,
}