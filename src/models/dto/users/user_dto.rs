use serde::Serialize;

use crate::models::entities::user::user_role::UserRole;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub profile_picture: Option<String>,
    pub role: UserRole,
}
