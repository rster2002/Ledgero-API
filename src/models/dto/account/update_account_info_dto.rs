use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountInfoDto<'a> {
    pub username: &'a str,
}
