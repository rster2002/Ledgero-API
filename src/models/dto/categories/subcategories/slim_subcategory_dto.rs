use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlimSubcategoryDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}
