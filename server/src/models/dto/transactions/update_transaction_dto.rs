use serde::Deserialize;

use crate::models::dto::transactions::new_split_dto::NewSplitDto;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionDto<'a> {
    pub description: &'a str,
    pub category_id: Option<&'a str>,
    pub subcategory_id: Option<&'a str>,
    pub external_account_id: Option<&'a str>,
    pub splits: Vec<NewSplitDto<'a>>,
}
