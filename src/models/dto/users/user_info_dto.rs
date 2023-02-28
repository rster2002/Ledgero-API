use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserInfoDto<'a> {
    pub username: &'a str,
}
