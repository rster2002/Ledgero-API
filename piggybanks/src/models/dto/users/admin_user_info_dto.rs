use crate::models::entities::user::user_role::UserRole;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AdminUserInfoDto<'a> {
    pub username: &'a str,
    pub role: UserRole,
}
