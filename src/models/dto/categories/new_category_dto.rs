use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCategoryDto {
    pub name: String,
    pub description: String,
    pub hex_color: String,
}
