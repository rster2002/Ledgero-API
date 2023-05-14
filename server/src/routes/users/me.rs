use rocket::http::Status;
use rocket::serde::json::Json;
use jumpdrive_auth::services::{PasswordHashService, TotpService};
use rand::Rng;

use crate::db_inner;
use crate::models::dto::account::enable_mfa_dto::EnableMfaDto;
use crate::models::dto::account::mfa_enabled_response_dto::MfaEnabledResponseDto;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::dto::users::update_user_password_dto::UpdateUserPasswordDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::dto::users::user_info_dto::UserInfoDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::users::shared_resolvers::{
    resolve_delete_user, resolve_update_user_info, resolve_update_user_password, resolve_user_by_id,
};
use crate::shared::{SharedBlobService, SharedPool};

#[get("/me")]
pub async fn get_me_info(pool: &SharedPool, user: JwtUserPayload) -> Result<Json<UserDto>> {
    resolve_user_by_id(pool, &user.uuid).await
}

#[patch("/me", data = "<body>")]
pub async fn update_me_info(
    pool: &SharedPool,
    blob_service: &SharedBlobService,
    user: JwtUserPayload,
    body: Json<UserInfoDto<'_>>,
) -> Result<()> {
    info!("{} updated their account info", user);
    resolve_update_user_info(
        pool,
        blob_service,
        &user.uuid,
        &AdminUserInfoDto {
            username: body.username,
            role: user.role,
            image_token: body.image_token,
        },
    )
    .await
}

#[patch("/me/password", data = "<body>")]
pub async fn update_me_password(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<UpdateUserPasswordDto<'_>>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT *
            FROM Users
            WHERE Id = $1;
        "#,
        user.uuid
    )
    .fetch_one(inner_pool)
    .await?;

    let valid_password = PasswordHashService::verify(record.passwordhash, body.old_password);
    if !valid_password {
        return Err(Status::Unauthorized.into());
    }

    info!("{} updated their password", user);
    resolve_update_user_password(
        pool,
        &user.uuid,
        &AdminUpdateUserPasswordDto {
            new_password: body.new_password,
        },
    )
    .await
}

#[delete("/me")]
pub async fn delete_me(pool: &SharedPool, user: JwtUserPayload) -> Result<()> {
    info!("{} deleted their own account", user);
    resolve_delete_user(pool, &user.uuid).await
}

#[patch("/me/enable-mfa", data="<body>")]
pub async fn enable_mfa_me(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<EnableMfaDto>,
) -> Result<Json<MfaEnabledResponseDto>> {
    let inner_pool = db_inner!(pool);
    let body = body.0;

    if body.secret_key.len() < 32 {
        return Err(Status::BadRequest.into());
    }

    TotpService::guard_code(&body.secret_key, &body.code)?;

    let backup_codes:[String; 8] = {
        let mut rng = rand::thread_rng();

        let backup_codes = [
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
            rng.gen_range(100000..=999999),
        ]
            .map(|code| code.to_string());

        backup_codes
    };

    sqlx::query!(
        r#"
            UPDATE Users
            SET mfaSecret = $2, mfaBackupCodes = $3
            WHERE Id = $1
        "#,
        user.uuid,
        body.secret_key,
        &backup_codes
    )
        .execute(inner_pool)
        .await?;

    Ok(Json(MfaEnabledResponseDto {
        backup_codes,
    }))
}

#[patch("/me/disable-mfa")]
pub async fn disable_mfa_me(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    sqlx::query!(
        r#"
            UPDATE Users
            SET mfaSecret = null, mfaBackupCodes = null
            WHERE Id = $1;
        "#,
        user.uuid
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}
