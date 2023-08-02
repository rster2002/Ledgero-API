use serde::Deserialize;
use serde_email::Email;

#[derive(Debug, Deserialize)]
pub struct RegisterUserDto<'a> {
    pub username: &'a str,
    pub email: Email,
    pub password: &'a str,
}
