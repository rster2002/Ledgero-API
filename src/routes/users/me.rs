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
use crate::services::password_hash_service::PasswordHashService;
use crate::shared::{SharedBlobService, SharedPool};
use rocket::http::Status;
use rocket::serde::json::Json;
use crate::db_inner;

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
    resolve_delete_user(pool, &user.uuid).await
}
