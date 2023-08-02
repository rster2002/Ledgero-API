use jumpdrive_auth::errors::TotpError;
use jumpdrive_auth::services::{PasswordHashService, TotpService};
use rocket::{Request, State};
use rocket::http::Status;
use rocket::serde::json::Json;

use crate::db_inner;
use crate::models::dto::auth::auth_response_dto::AuthResponseDto;
use crate::models::dto::auth::auth_response_dto::jwt_access_token_payload::JwtAccessTokenPayload;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::entities::grant::Grant;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_refresh_payload::JwtRefreshPayload;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::services::rate_limiter::RateLimiter;
use crate::shared::{SharedJwtService, SharedPool};

#[post("/login", data = "<body>")]
pub async fn perform_login<'a>(
    pool: &'a SharedPool,
    jwt_service: &'a SharedJwtService,
    rate_limiter: &State<RateLimiter>,
    body: Json<LoginUserDto<'a>>,
) -> Result<Json<AuthResponseDto>> {
    let pool = db_inner!(pool);
    let body = body.0;

    rate_limiter.limit(&*body.username, 1)?;

    let mut db_transaction = pool.begin().await?;

    info!("Login attempt for '{}'", body.username);

    let user = sqlx::query!(
        r#"
            SELECT *
            FROM Users
            WHERE Username = $1;
        "#,
        body.username
    )
        .fetch_optional(&mut *db_transaction)
        .await?;

    debug!("Checking if user with username '{}' exists", body.username);
    let Some(user) = user else {
        info!("No user exists with username '{}'", body.username);
        return Err(Status::Unauthorized.into());
    };

    let valid_password = PasswordHashService::verify(user.password_hash, body.password);
    if !valid_password {
        info!("The password for user '{}' was incorrect", body.username);
        return Err(Status::Unauthorized.into());
    }

    if let Some(mfa_secret) = user.mfa_secret {
        let Some(mfa_code) = body.mfa_code else {
            return Ok(Json(AuthResponseDto::TwoFAChallenge));
        };

        let valid_code = TotpService::validate_code(mfa_secret, mfa_code)?;

        if !valid_code {
            let mut backup_codes = user.mfa_backup_codes.unwrap();

            let found_backup_code_index = backup_codes
                .iter()
                .enumerate()
                .find(|(i, x)| x == &mfa_code)
                .map(|(i, _)| i);

            let Some(index) = found_backup_code_index else {
                return Err(TotpError::InvalidOneTimePassword.into());
            };

            backup_codes.swap_remove(index);

            sqlx::query!(
                r#"
                    UPDATE users
                    SET mfa_backup_codes = $2
                    WHERE id = $1;
                "#,
                user.id,
                &backup_codes
            )
                .execute(&mut *db_transaction)
                .await?;
        }
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

    grant.create(&mut *db_transaction).await?;
    db_transaction.commit().await?;

    info!("Successfully logged in '{}'", body.username);
    Ok(Json(AuthResponseDto::JwtAccessToken(JwtAccessTokenPayload {
        access_token: jwt,
        refresh_token: refresh,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds(),
    })))
}
