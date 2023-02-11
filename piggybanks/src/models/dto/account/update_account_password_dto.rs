use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountPasswordDto<'a> {
    pub old_password: &'a str,
    pub new_password: &'a str,
}
