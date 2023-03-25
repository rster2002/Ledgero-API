use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCategoryDto<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub hex_color: &'a str,
}
