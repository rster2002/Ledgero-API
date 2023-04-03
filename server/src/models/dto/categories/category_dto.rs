use serde::Serialize;

use crate::models::dto::categories::subcategories::subcategory_dto::SubcategoryDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
    pub amount: i64,
    pub subcategories: Vec<SubcategoryDto>,
}
