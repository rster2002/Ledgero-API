use std::env;
use jumpdrive_auth::services::PasswordHashService;
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::registration_enabled_dto::RegistrationEnabledDto;
use crate::models::entities::user::User;
use crate::models::entities::user::user_role::UserRole;
use crate::shared::SharedPool;
use crate::prelude::*;

#[get("/register")]
pub async fn registration_enabled() -> Json<RegistrationEnabledDto> {
    Json(RegistrationEnabledDto {
        enabled: env::var("DISABLE_REGISTRATION").is_err(),
    })
}

#[post("/register", data = "<body>")]
pub async fn register(pool: &SharedPool, body: Json<RegisterUserDto<'_>>) -> Result<Status> {
    if env::var("DISABLE_REGISTRATION").is_ok() {
        return HttpError::new(403)
            .message("Registration is disabled")
            .into();
    }

    let pool = db_inner!(pool);
    let body = body.0;

    if body.username.len() < 4 {
        return Err(HttpError::new(400)
            .message("Username has to have at least four characters")
            .into());
    }

    if body.password.len() < 8 {
        return Err(HttpError::new(400)
            .message("Password has to have at least four characters")
            .into());
    }

    let password_hash = PasswordHashService::create_new_hash(body.password);
    let user = User {
        id: Uuid::new_v4().to_string(),
        username: body.username.to_string(),
        password_hash,
        role: UserRole::User,
    };

    user.create(pool).await?;

    Ok(Status::Accepted)
}
