use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct BulkUpdateTransactionCategoriesDto {
    pub transactions: Vec<String>,
    pub category_id: Option<String>,
    pub subcategory_id: Option<String>,
}
