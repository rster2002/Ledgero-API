/// Shared resolvers. Contains most of the logic for the admin and me endpoints, but don't contain
/// things like access control checking etc.
mod shared_resolvers;

/// User endpoints for admins allowing to modify the other user's information.
pub mod admin;

/// User endpoints for the current user.
pub mod me;

use rocket::http::Status;
use crate::models::dto::users::new_user_dto::NewUserDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::entities::user::user_role::UserRole;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;
use crate::utils::guard_role::guard_role;
use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;
use crate::error::http_error::HttpError;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::routes::users::admin::*;
use crate::routes::users::me::*;

pub fn create_user_routes() -> Vec<Route> {
    routes![
        admin_get_users,
        admin_create_user,
        admin_get_user_by_id,
        admin_update_user_information,
        admin_update_user_password,
        admin_delete_user,
        get_me_info,
        update_me_info,
        update_me_password,
        delete_me,
    ]
}
