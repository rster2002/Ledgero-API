use rocket::Route;

use crate::routes::users::admin::*;
use crate::routes::users::me::*;

/// Shared resolvers. Contains most of the logic for the admin and me endpoints, but don't contain
/// things like access control checking etc.
mod shared_resolvers;

/// User endpoints for admins allowing to modify the other user's information.
pub mod admin;

/// User endpoints for the current user.
pub mod me;

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
