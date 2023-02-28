use crate::models::entities::user::user_role::UserRole;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UserInfoDto<'a> {
    pub username: &'a str,
}
