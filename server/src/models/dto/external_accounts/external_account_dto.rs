use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAccountDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
    pub default_category_id: Option<String>,
    pub default_subcategory_id: Option<String>,
}
