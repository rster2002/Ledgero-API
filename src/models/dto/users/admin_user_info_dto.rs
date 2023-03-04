use serde::Deserialize;

use crate::models::entities::user::user_role::UserRole;

#[derive(Debug, Deserialize)]
pub struct AdminUserInfoDto<'a> {
    pub username: &'a str,
    pub image_token: Option<&'a str>,
    pub role: UserRole,
}
