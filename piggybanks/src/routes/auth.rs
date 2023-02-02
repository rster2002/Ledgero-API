use crate::error::http_error::HttpError;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::dto::auth::jwt_response_dto::JwtResponseDto;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::models::service::jwt_service::JwtService;
use crate::models::service::password_hash_service::PasswordHashService;
use crate::prelude::*;
use crate::shared_types::{SharedJwtService, SharedPool};
use chrono::format::Numeric::Month;
use chrono::{Duration, Months, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use sqlx::{Pool, Postgres};
use std::ops::Add;
use uuid::Uuid;

pub fn create_auth_routes() -> Vec<Route> {
    routes![register, login, refresh, revoke,]
}

#[post("/register", data = "<body>")]
pub async fn register(
    body: Json<RegisterUserDto<'_>>,
    pool: &State<Pool<Postgres>>,
) -> Result<Status> {
    let body = body.0;

    let password_hash = PasswordHashService::create_new_hash(body.password);
    let user = User::new(body.username, password_hash);

    user.create(pool).await?;

    Ok(Status::Accepted)
}

#[post("/login", data = "<body>")]
async fn login<'a>(
    body: Json<LoginUserDto<'a>>,
    pool: &'a SharedPool,
    jwt_service: &'a SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
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

    Ok(Json(JwtResponseDto {
        access_token: jwt,
        refresh_token: refresh,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/refresh", data = "<body>")]
async fn refresh(
    body: Json<JwtRefreshDto<'_>>,
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
    let body = body.0;

    let (_, access_payload) =
        jwt_service.decode_access_token_unchecked::<JwtUserPayload>(body.access_token)?;
    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;

    let Some(_) = User::by_id(pool, &access_payload.uuid).await? else {
        return Err(
            HttpError::from_status(Status::NotFound)
                .message("No user with the give id was found. The user might have been deleted")
                .into()
        );
    };

    let Some(mut grant) = Grant::by_id(pool, refresh_payload.grant_id).await? else {
        return Err(
            HttpError::from_status(Status::Unauthorized)
                .message("The given refresh token has been revoked")
                .into()
        );
    };

    grant.set_expire_at(Utc::now() + Months::new(3));
    grant.update(pool).await?;

    let access_token = jwt_service.create_access_token(&access_payload)?;
    let refresh_token = jwt_service.create_refresh_token(&grant)?;

    Ok(Json(JwtResponseDto {
        access_token,
        refresh_token,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/revoke", data = "<body>")]
async fn revoke(
    body: Json<RevokeDto>,
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
) -> Result<()> {
    let body = body.0;
    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;

    Grant::delete_by_id(pool, refresh_payload.grant_id).await?;

    Ok(())
}
