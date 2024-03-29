use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;
use jumpdrive_auth::services::PasswordHashService;

use crate::db_inner;
use crate::models::dto::users::admin_update_user_password_dto::AdminUpdateUserPasswordDto;
use crate::models::dto::users::admin_user_info_dto::AdminUserInfoDto;
use crate::models::dto::users::new_user_dto::NewUserDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::entities::user::user_role::UserRole;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::users::shared_resolvers::{
    resolve_delete_user, resolve_update_user_info, resolve_update_user_password, resolve_user_by_id,
};
use crate::shared::{SharedBlobService, SharedPool};
use crate::utils::guard_role::guard_user_payload;

#[get("/")]
pub async fn admin_get_users(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<Json<Vec<UserDto>>> {
    guard_user_payload(&user, UserRole::System)?;

    let inner_pool = db_inner!(pool);

    let records = sqlx::query!(
        r#"
            SELECT id, username, profile_image, role
            FROM users;
        "#
    )
    .fetch_all(inner_pool)
    .await?;

    let users = records
        .into_iter()
        .map(|record| UserDto {
            id: record.id,
            username: record.username,
            profile_picture: record.profile_image,
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
    guard_user_payload(&user, UserRole::System)?;

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

    info!(
        "{} created a new user '{}' ({}, {})",
        user, new_user.username, new_user.role, new_user.id
    );
    admin_get_user_by_id(pool, user, uuid.to_string()).await
}

#[get("/<id>")]
pub async fn admin_get_user_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<UserDto>> {
    guard_user_payload(&user, UserRole::System)?;
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
    guard_user_payload(&user, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_update_user_info(pool, blob_service, &id, &body).await?;

    info!("{} updated the information of user '{}'", user, id);
    admin_get_user_by_id(pool, user, id).await
}

#[patch("/<id>/password", data = "<body>")]
pub async fn admin_update_user_password(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<AdminUpdateUserPasswordDto<'_>>,
) -> Result<Status> {
    guard_user_payload(&user, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_update_user_password(pool, &id, &body).await?;

    info!("{} updated the password of user '{}'", user, id);
    Ok(Status::Accepted)
}

#[delete("/<id>")]
pub async fn admin_delete_user(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    guard_user_payload(&user, UserRole::System)?;

    resolve_user_by_id(pool, &id).await?;

    resolve_delete_user(pool, &id).await?;

    info!("{} deleted user '{}'", user, id);
    Ok(())
}
