use rocket::serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSetCategoryDto {
    pub category_id: Option<String>,
    pub subcategory_id: Option<String>,
}
