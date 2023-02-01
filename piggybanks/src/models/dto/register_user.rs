use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterUserDto<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
