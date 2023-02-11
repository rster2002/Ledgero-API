use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPasswordDto<'a> {
    pub new_password: &'a str,
}
