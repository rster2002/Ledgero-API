use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginUserDto<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub mfa_code: Option<&'a str>,
}
