use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExternalAccountNameDto<'a> {
    pub name: &'a str,
}
