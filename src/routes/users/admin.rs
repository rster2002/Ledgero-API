use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::db_inner;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::dto::users::new_user_dto::NewUserDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::entities::user::User;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::users::shared_resolvers::{
    resolve_delete_user, resolve_update_user_info, resolve_update_user_password, resolve_user_by_id,
};
use crate::services::password_hash_service::PasswordHashService;
use crate::shared::{SharedBlobService, SharedPool};
use crate::utils::guard_role::guard_role;

#[get("/")]
pub async fn admin_get_users(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<UserDto>>> {
    guard_role(&user.role, UserRole::System)?;

    let inner_pool = db_inner!(pool);

    let records = sqlx::query!(
        r#"
            SELECT Id, Username, ProfileImage, Role
            FROM Users;
        "#
    )
    .fetch_all(inner_pool)
    .await?;

    let users = records
        .into_iter()
        .map(|record| UserDto {
            id: record.id,
            username: record.username,
            profile_picture: record.profileimage,
            role: UserRole::from(record.role),
        })
        .collect();

    Ok(Json(users))
}

#[post("/", data = "<body>")]
pub async fn admin_create_user(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewUserDto<'_>>,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;

    let inner_pool = db_inner!(pool);
    let body = body.0;

    let password_hash = PasswordHashService::create_new_hash(body.password);

    let uuid = Uuid::new_v4();

    let new_user = User {
        id: uuid.to_string(),
        username: body.username.to_string(),
        password_hash,
        role: body.role,
    };

    new_user.create(inner_pool).await?;

    admin_get_user_by_id(pool, user, uuid.to_string()).await
}

#[get("/<id>")]
pub async fn admin_get_user_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;
    resolve_user_by_id(pool, &id).await
}

#[patch("/<id>", data = "<body>")]
pub async fn admin_update_user_information(
    pool: &SharedPool,
    blob_service: &SharedBlobService,
    user: JwtUserPayload,
    id: String,
    body: Json<AdminUserInfoDto<'_>>,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_update_user_info(pool, blob_service, &id, &body).await?;

    admin_get_user_by_id(pool, user, id).await
}

#[patch("/<id>/password", data = "<body>")]
pub async fn admin_update_user_password(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<AdminUpdateUserPasswordDto<'_>>,
) -> Result<Status> {
    guard_role(&user.role, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_update_user_password(pool, &id, &body).await?;

    Ok(Status::Accepted)
}

#[delete("/<id>")]
pub async fn admin_delete_user(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    guard_role(&user.role, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_delete_user(pool, &id).await?;

    Ok(())
}
