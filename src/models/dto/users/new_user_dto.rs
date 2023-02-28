use crate::models::entities::user::user_role::UserRole;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NewUserDto<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub role: UserRole,
}
