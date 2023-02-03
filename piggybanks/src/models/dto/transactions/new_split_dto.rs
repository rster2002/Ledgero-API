use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSplitDto {
    pub description: String,
    pub amount: i64,
    pub category_id: Option<String>,
}
