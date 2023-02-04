use serde::Serialize;
use crate::models::dto::categories::category_dto::CategoryDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitDto {
    pub id: String,
    pub description: String,
    pub amount: i64,
    pub category: Option<CategoryDto>,
}
