use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExternalAccountDto {
    pub name: String,
    pub description: String,
    pub default_category_id: Option<String>,
}
