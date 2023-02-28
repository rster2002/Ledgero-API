use crate::error::http_error::HttpError;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::dto::auth::jwt_response_dto::JwtResponseDto;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;

use crate::prelude::*;
use crate::shared_types::{SharedJwtService, SharedPool};

use crate::models::entities::user::user_role::UserRole;
use chrono::{Months, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Route, State};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::services::password_hash_service::PasswordHashService;

pub fn create_auth_routes() -> Vec<Route> {
    routes![register, login, refresh, revoke,]
}

#[post("/register", data = "<body>")]
pub async fn register(
    pool: &State<Pool<Postgres>>,
    body: Json<RegisterUserDto<'_>>,
) -> Result<Status> {
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
async fn login<'a>(
    pool: &'a SharedPool,
    body: Json<LoginUserDto<'a>>,
    jwt_service: &'a SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
    let pool = pool.inner();
    let body = body.0;

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

    let Some(user) = user else {
        return Err(Status::Unauthorized.into());
    };

    let valid_password = PasswordHashService::verify(user.passwordhash, body.password);
    if !valid_password {
        return Err(Status::Unauthorized.into());
    }

    let grant = Grant::new(&user.id);

    let user_payload = JwtUserPayload {
        uuid: user.id.to_string(),
        username: user.username.to_string(),
        role: UserRole::from(user.role),
    };

    let jwt = jwt_service.create_access_token(&user_payload)?;
    let refresh = jwt_service.create_refresh_token(&grant.id)?;

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
    pool: &SharedPool,
    body: Json<JwtRefreshDto<'_>>,
    jwt_service: &SharedJwtService,
) -> Result<Json<JwtResponseDto>> {
    let pool = pool.inner();
    let body = body.0;

    let (_, access_payload) =
        jwt_service.decode_access_token_unchecked::<JwtUserPayload>(body.access_token)?;
    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;

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

    sqlx::query!(
        r#"
            UPDATE Grants
            SET ExpireAt = $2
            WHERE Id = $1;
        "#,
        grant.id,
        (Utc::now() + Months::new(3)).to_rfc3339()
    )
    .execute(pool)
    .await?;

    let access_token = jwt_service.create_access_token(&access_payload)?;
    let refresh_token = jwt_service.create_refresh_token(&grant.id)?;

    Ok(Json(JwtResponseDto {
        access_token,
        refresh_token,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds() as u32,
    }))
}

#[post("/revoke", data = "<body>")]
async fn revoke(
    pool: &SharedPool,
    body: Json<RevokeDto>,
    jwt_service: &SharedJwtService,
) -> Result<()> {
    let body = body.0;
    let refresh_payload = jwt_service.decode_refresh_token(body.refresh_token)?;

    Grant::delete_by_id(pool, refresh_payload.grant_id).await?;

    Ok(())
}
