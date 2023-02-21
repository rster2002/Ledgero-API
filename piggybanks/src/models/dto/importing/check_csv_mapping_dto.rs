use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCsvMappingDto {
    pub account_iban: String,
    pub date: String,
    pub follow_number: String,
    pub description: String,
    pub amount: i64,
    pub external_account_name: String,
}
