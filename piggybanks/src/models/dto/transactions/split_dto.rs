use crate::models::dto::categories::category_dto::CategoryDto;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitDto {
    pub id: String,
    pub description: String,
    pub amount: i64,
    pub category: Option<CategoryDto>,
}
