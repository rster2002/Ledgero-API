use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExternalAccountDto<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub hex_color: &'a str,
    pub default_category_id: Option<&'a str>,
    pub default_subcategory_id: Option<&'a str>,
}
