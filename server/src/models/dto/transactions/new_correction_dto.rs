use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCorrectionDto {
    pub amount: i64,
    pub description: String,
    pub bank_account_id: String,
    pub category_id: Option<String>,
    pub subcategory_id: Option<String>,
}
