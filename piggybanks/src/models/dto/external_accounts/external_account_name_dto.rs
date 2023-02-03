use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAccountNameDto {
    pub id: String,
    pub name: String,
    pub parent_external_account: String,
}
