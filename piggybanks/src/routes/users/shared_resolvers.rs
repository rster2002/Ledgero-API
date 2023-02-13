use rocket::http::Status;
use rocket::serde::json::Json;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::models::service::password_hash_service::PasswordHashService;
use crate::shared_types::SharedPool;
use crate::prelude::*;

pub async fn resolve_user_by_id(
    pool: &SharedPool,
    id: &String,
) -> Result<Json<UserDto>> {
    let inner_pool = pool.inner();

    let record = sqlx::query!(
        r#"
            SELECT Id, Username, Role
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
        role: UserRole::from(record.role),
    }))
}

pub async fn resolve_update_user_info(
    pool: &SharedPool,
    id: &String,
    body: &AdminUserInfoDto<'_>,
) -> Result<()> {
    let inner_pool = pool.inner();

    let role_str: &str = body.role.into();
    let record = sqlx::query!(
        r#"
            UPDATE Users
            SET Username = $2,
            Role = $3
            WHERE Id = $1;
        "#,
        id,
        &body.username,
        role_str
    )
        .execute(inner_pool)
        .await?;

    Ok(())
}

pub async fn resolve_update_user_password(
    pool: &SharedPool,
    id: &String,
    body: &AdminUpdateUserPasswordDto<'_>
) -> Result<()> {
    let inner_pool = pool.inner();

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

pub async fn resolve_delete_user(
    pool: &SharedPool,
    id: &String
) -> Result<()> {
    let inner_pool = pool.inner();

    let record = sqlx::query!(
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
