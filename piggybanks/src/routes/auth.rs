use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use sqlx::{Pool, Postgres};
use crate::models::dto::jwt_response::JwtResponse;
use crate::models::dto::login_user_dto::LoginUserDto;
use crate::models::dto::register_user::RegisterUserDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::models::service::jwt_service::JwtService;
use crate::models::service::password_hash_service::PasswordHashService;
use crate::prelude::*;

pub fn create_auth_routes() -> Vec<Route> {
    routes![register, login]
}

#[post("/register", data = "<body>")]
pub async fn register(body: Json<RegisterUserDto<'_>>, pool: &State<Pool<Postgres>>) -> Result<Status> {
    let body = body.0;

    let password_hash = PasswordHashService::create_new_hash(body.password);
    let user = User::new(body.username, password_hash);

    user.create(pool).await?;

    Ok(Status::Accepted)
}

#[post("/login", data = "<body>")]
async fn login<'a>(
    body: Json<LoginUserDto<'a>>,
    pool: &'a State<Pool<Postgres>>,
    jwt_service: &'a State<JwtService>,
) -> Result<Json<JwtResponse>> {
    let body = body.0;

    let user = User::by_username(pool, body.username).await?;

    let Some(user) = user else {
        return Err(Status::Unauthorized.into());
    };

    let valid_password = PasswordHashService::verify(user.password_hash, body.password);
    if !valid_password {
        return Err(Status::Unauthorized.into());
    }

    let grant = Grant::new(&user.id);

    let user_payload = JwtUserPayload {
        uuid: user.id.to_string(),
        username: user.username.to_string(),
    };

    let jwt = jwt_service.create_access_token(&user_payload)?;
    let refresh = jwt_service.create_refresh_token(&grant)?;

    grant.create(pool).await?;

    Ok(Json(JwtResponse {
        access_token: jwt,
        refresh_token: refresh,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/refresh", data = "<body>")]
async fn refresh() -> Result<JwtResponse> {
    todo!()
}
