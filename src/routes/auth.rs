use chrono::{Months, Utc};
use rocket::{Route, State};
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::dto::auth::jwt_response_dto::JwtResponseDto;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::User;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::services::password_hash_service::PasswordHashService;
use crate::shared::{SharedJwtService, SharedPool};

pub fn create_auth_routes() -> Vec<Route> {
    routes![register, login, refresh, revoke,]
}

#[post("/register", data = "<body>")]
pub async fn register(
    pool: &SharedPool,
    body: Json<RegisterUserDto<'_>>,
) -> Result<Status> {
    let pool = db_inner!(pool);
    let body = body.0;

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

#[post("/login", data = "<body>")]
pub async fn login<'a>(
    pool: &'a SharedPool,
    body: Json<LoginUserDto<'a>>,
    jwt_service: &'a SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
    let pool = db_inner!(pool);
    let body = body.0;

    info!("Login attempt for '{}'", body.username);

    let user = sqlx::query!(
        r#"
            SELECT *
            FROM Users
            WHERE Username = $1;
        "#,
        body.username
    )
    .fetch_optional(pool)
    .await?;

    debug!("Checking if user with username '{}' exists", body.username);
    let Some(user) = user else {
        info!("No user exists with username '{}'", body.username);
        return Err(Status::Unauthorized.into());
    };

    let valid_password = PasswordHashService::verify(user.passwordhash, body.password);
    if !valid_password {
        info!("The password for user '{}' was incorrect", body.username);
        return Err(Status::Unauthorized.into());
    }

    let grant = Grant::new(&user.id);

    let user_payload = JwtUserPayload {
        uuid: user.id.to_string(),
        username: user.username.to_string(),
        role: UserRole::from(user.role),
    };

    debug!("Generating a new JWT access token for '{}'", body.username);
    let jwt = jwt_service.create_access_token(&user_payload)?;

    debug!("Generating a new JWT refresh token for '{}'", body.username);
    let refresh = jwt_service.create_refresh_token(&grant.id)?;

    grant.create(pool).await?;

    info!("Successfully logged in '{}'", body.username);
    Ok(Json(JwtResponseDto {
        access_token: jwt,
        refresh_token: refresh,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/refresh", data = "<body>")]
pub async fn refresh(
    pool: &SharedPool,
    body: Json<JwtRefreshDto<'_>>,
    jwt_service: &SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
    let pool = db_inner!(pool);
    let body = body.0;

    let (_, access_payload) =
        jwt_service.decode_access_token_unchecked::<JwtUserPayload>(body.access_token)?;
    info!("Token refresh attempt for '{}'", access_payload.username);

    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;
    debug!("Attempting to refresh using grant id '{}'", refresh_payload.grant_id);

    let user = sqlx::query!(
        r#"
            SELECT Id
            FROM Users
            WHERE Id = $1;
        "#,
        access_payload.uuid
    )
    .fetch_optional(pool)
    .await?;

    let Some(_) = user else {
        return Err(
            HttpError::from_status(Status::NotFound)
                .message("No user with the give id was found. The user might have been deleted")
                .into()
        );
    };

    let grant = sqlx::query!(
        r#"
            SELECT *
            FROM Grants
            WHERE Id = $1;
        "#,
        refresh_payload.grant_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(grant) = grant else {
        return Err(
            HttpError::from_status(Status::Unauthorized)
                .message("The given refresh token has been revoked")
                .into()
        );
    };

    let new_expire_at = (Utc::now() + Months::new(3)).to_rfc3339();

    debug!("Updating grant '{}' with new expire time '{}'", grant.id, new_expire_at);
    sqlx::query!(
        r#"
            UPDATE Grants
            SET ExpireAt = $2
            WHERE Id = $1;
        "#,
        grant.id,
        new_expire_at
    )
    .execute(pool)
    .await?;

    let access_token = jwt_service.create_access_token(&access_payload)?;
    let refresh_token = jwt_service.create_refresh_token(&grant.id)?;

    info!("Successfully refreshed JWT access token for '{}'", access_payload.username);
    Ok(Json(JwtResponseDto {
        access_token,
        refresh_token,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/revoke", data = "<body>")]
pub async fn revoke(
    pool: &SharedPool,
    body: Json<RevokeDto>,
    jwt_service: &SharedJwtService,
) -> Result<()> {
    let pool = db_inner!(pool);

    let body = body.0;
    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;

    debug!("Revoking grant with id '{}'", refresh_payload.grant_id);
    Grant::delete_by_id(pool, refresh_payload.grant_id).await?;

    Ok(())
}
