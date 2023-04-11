use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoDto<'a> {
    pub username: &'a str,
    pub image_token: Option<&'a str>,
}
