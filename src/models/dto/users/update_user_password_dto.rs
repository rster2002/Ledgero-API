use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPasswordDto<'a> {
    pub old_password: &'a str,
    pub new_password: &'a str,
}
