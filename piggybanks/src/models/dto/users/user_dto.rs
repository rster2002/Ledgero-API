use crate::models::entities::user::user_role::UserRole;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub role: UserRole,
}
