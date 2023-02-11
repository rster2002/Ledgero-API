use rocket::Route;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared_types::SharedPool;

pub fn create_account_routes() -> Vec<Route> {
    routes![
        get_account_info,
        update_account_info,
        update_account_password,
    ]
}

#[get("/me")]
pub async fn get_account_info(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<UserDto> {
    todo!()
}

#[patch("/me")]
pub async fn update_account_info() -> Result<UserDto> {
    todo!()
}

#[patch("/me/password")]
pub async fn update_account_password() -> Result<UserDto> {
    todo!()
}

