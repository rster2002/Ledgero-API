use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct MoveBetweenCategoriesDto {
    pub from_category_id: String,
    pub from_subcategory_id: Option<String>,
    pub to_category_id: String,
    pub to_subcategory_id: Option<String>,
    pub amount: u32,
}
