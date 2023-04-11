use rocket::serde::json::Json;
use jumpdrive_auth::services::PasswordHashService;

use crate::db_inner;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;
use crate::shared::{SharedBlobService, SharedPool};

pub async fn resolve_user_by_id(pool: &SharedPool, id: &String) -> Result<Json<UserDto>> {
    let inner_pool = db_inner!(pool);

    let record = sqlx::query!(
        r#"
            SELECT Id, Username, ProfileImage, Role
            FROM Users
            WHERE Id = $1;
        "#,
        id
    )
    .fetch_one(inner_pool)
    .await?;

    Ok(Json(UserDto {
        id: record.id,
        username: record.username,
        profile_picture: record.profileimage,
        role: UserRole::from(record.role),
    }))
}

pub async fn resolve_update_user_info(
    pool: &SharedPool,
    blob_service: &SharedBlobService,
    user_id: &String,
    body: &AdminUserInfoDto<'_>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let blob_service = blob_service.read().await;
    let image_token = blob_service
        .confirm_optional(user_id, inner_pool, body.image_token)
        .await?;

    let role_str: &str = body.role.into();
    let _record = sqlx::query!(
        r#"
            UPDATE Users
            SET Username = $2, Role = $3, ProfileImage = $4
            WHERE Id = $1;
        "#,
        user_id,
        &body.username,
        role_str,
        image_token
    )
    .execute(inner_pool)
    .await?;

    Ok(())
}

pub async fn resolve_update_user_password(
    pool: &SharedPool,
    id: &String,
    body: &AdminUpdateUserPasswordDto<'_>,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let new_hash = PasswordHashService::create_new_hash(body.new_password);

    sqlx::query!(
        r#"
            UPDATE Users
            SET passwordHash = $2
            WHERE Id = $1
        "#,
        id,
        new_hash
    )
    .execute(inner_pool)
    .await?;

    Ok(())
}

pub async fn resolve_delete_user(pool: &SharedPool, id: &String) -> Result<()> {
    let inner_pool = db_inner!(pool);

    let _record = sqlx::query!(
        r#"
            DELETE FROM Users
            WHERE Id = $1;
        "#,
        id
    )
    .execute(inner_pool)
    .await?;

    Ok(())
}
