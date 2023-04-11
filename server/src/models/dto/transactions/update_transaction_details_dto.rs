use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransactionDetailsDto<'a> {
    pub description: &'a str,
    pub category_id: Option<&'a str>,
    pub subcategory_id: Option<&'a str>,
    pub external_account_id: Option<&'a str>,
}
