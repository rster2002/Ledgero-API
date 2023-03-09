use rocket::serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSetCategoryDto<'a> {
    pub category_id: Option<&'a str>,
    pub subcategory_id: Option<&'a str>,
}
