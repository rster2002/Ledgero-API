use serde::Deserialize;

use crate::models::entities::user::user_role::UserRole;

#[derive(Debug, Deserialize)]
pub struct NewUserDto<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub role: UserRole,
}
