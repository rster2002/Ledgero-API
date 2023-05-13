use std::env;

use jumpdrive_auth::services::TotpService;
use rocket::Route;
use rocket::serde::json::Json;

use crate::models::dto::auth::random_mfa_secret_key_dto::RandomMfaSecretKeyDto;
use crate::prelude::*;
use crate::routes::auth::login::*;
use crate::routes::auth::refresh_token::*;
use crate::routes::auth::registration::*;
use crate::routes::auth::revoke_token::*;

pub mod login;
pub mod registration;
pub mod refresh_token;
pub mod revoke_token;

pub fn create_auth_routes() -> Vec<Route> {
    routes![
        registration_enabled,
        register,
        perform_login,
        refresh,
        revoke,
        revoke_all,
        get_random_mfa_secret_key,
    ]
}

#[get("/random-mfa-secret-key")]
pub async fn get_random_mfa_secret_key() -> Result<Json<RandomMfaSecretKeyDto>> {
    Ok(Json(RandomMfaSecretKeyDto {
        secret_key: TotpService::generate_secret_key(),
    }))
}
