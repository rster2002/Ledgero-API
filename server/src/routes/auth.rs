use std::env;
use chrono::{Months, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::Route;

use uuid::Uuid;
use jumpdrive_auth::models::jwt::JwtRefreshPayload;
use jumpdrive_auth::services::{PasswordHashService, TotpService};

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::dto::auth::auth_response_dto::AuthResponseDto;
use crate::models::dto::auth::auth_response_dto::jwt_access_token_payload::JwtAccessTokenPayload;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::registration_enabled_dto::RegistrationEnabledDto;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::user_role::UserRole;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared::{SharedJwtService, SharedPool};

pub fn create_auth_routes() -> Vec<Route> {
    routes![
        registration_enabled,
        register,
        login,
        refresh,
        revoke,
        revoke_all,
    ]
}

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

#[post("/login", data = "<body>")]
pub async fn login<'a>(
    pool: &'a SharedPool,
    jwt_service: &'a SharedJwtService,
    body: Json<LoginUserDto<'a>>,
) -> Result<Json<AuthResponseDto>> {
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

    if let Some(mfa_secret) = user.mfasecret {
        let Some(mfa_code) = body.mfa_code else {
            return Ok(Json(AuthResponseDto::TwoFAChallenge));
        };

        TotpService::guard_code(mfa_secret, mfa_code)?;
    }

    let grant = Grant::new(&user.id);

    let user_payload = JwtUserPayload {
        uuid: user.id.to_string(),
        username: user.username.to_string(),
        role: UserRole::from(user.role),
    };

    debug!("Generating a new JWT access token for '{}'", body.username);
    let jwt = jwt_service.create_access_token(
        &user.id,
        &user_payload
    )?;

    debug!("Generating a new JWT refresh token for '{}'", body.username);
    let refresh = jwt_service.create_refresh_token(
        &user.id,
        JwtRefreshPayload {
            grant_id: grant.id.to_string(),
        },
    )?;

    grant.create(pool).await?;

    info!("Successfully logged in '{}'", body.username);
    Ok(Json(AuthResponseDto::JwtAccessToken(JwtAccessTokenPayload {
        access_token: jwt,
        refresh_token: refresh,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds(),
    })))
}

#[post("/refresh", data = "<body>")]
pub async fn refresh(
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
    body: Json<JwtRefreshDto<'_>>,
) -> Result<Json<AuthResponseDto>> {
    let pool = db_inner!(pool);
    let body = body.0;

    let (_, access_payload) =
        jwt_service.decode_access_token_unchecked::<JwtUserPayload>(body.access_token)?;
    info!("Token refresh attempt for '{}'", access_payload.username);

    let refresh_payload: JwtRefreshPayload = jwt_service.decode_refresh_token(body.refresh_token)?;
    debug!(
        "Attempting to refresh using grant id '{}'",
        refresh_payload.grant_id
    );

    trace!("Checking if the user for the grant still exists");
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
        debug!("No user was found for grant id '{}'", refresh_payload.grant_id);
        return Err(
            HttpError::from_status(Status::NotFound)
                .message("No user with the give id was found. The user might have been deleted")
                .into()
        );
    };

    trace!("Checking if the grant exists");
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
        debug!("No grant found with id '{}'", refresh_payload.grant_id);
        return Err(
            HttpError::from_status(Status::Unauthorized)
                .message("The given refresh token has been revoked")
                .into()
        );
    };

    let new_grant_id = Uuid::new_v4().to_string();
    let new_expire_at = (Utc::now() + Months::new(3)).to_rfc3339();

    debug!(
        "Updating grant '{}' with new id '{}' and expire time '{}'",
        grant.id, new_grant_id, new_expire_at
    );
    sqlx::query!(
        r#"
            UPDATE Grants
            SET Id = $2, ExpireAt = $3
            WHERE Id = $1;
        "#,
        grant.id,
        new_grant_id,
        new_expire_at
    )
    .execute(pool)
    .await?;

    trace!("Generating new access- and refresh tokens");
    let access_token = jwt_service.create_access_token(
        &access_payload.uuid,
        &access_payload
    )?;

    let refresh_token = jwt_service.create_refresh_token(
        &access_payload.uuid,
        JwtRefreshPayload {
            grant_id: new_grant_id,
        },
    )?;

    info!(
        "Successfully refreshed JWT access token for '{}'",
        access_payload.username
    );
    Ok(Json(AuthResponseDto::JwtAccessToken(JwtAccessTokenPayload {
        access_token,
        refresh_token,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds(),
    })))
}

#[post("/revoke", data = "<body>")]
pub async fn revoke(
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
    body: Json<RevokeDto>,
) -> Result<()> {
    let pool = db_inner!(pool);

    let body = body.0;
    let refresh_payload: JwtRefreshPayload = jwt_service.decode_refresh_token(body.refresh_token)?;

    debug!("Revoking grant with id '{}'", refresh_payload.grant_id);
    Grant::delete_by_id(pool, refresh_payload.grant_id).await?;

    Ok(())
}

#[put("/revoke-all")]
pub async fn revoke_all(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    debug!("Logging user {} out everywhere", user);
    sqlx::query!(
        r#"
            DELETE FROM Grants
            WHERE UserId = $1;
        "#,
        user.uuid
    )
        .execute(inner_pool)
        .await?;

    debug!("Logged out user {} everywhere", user);
    Ok(())
}
