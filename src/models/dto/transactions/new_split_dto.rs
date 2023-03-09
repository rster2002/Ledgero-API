use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSplitDto<'a> {
    pub description: &'a str,
    pub amount: i64,
    pub category_id: Option<&'a str>,
    pub subcategory_id: Option<&'a str>,
}
