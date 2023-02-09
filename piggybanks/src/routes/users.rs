use crate::models::dto::users::new_user_dto::NewUserDto;
use crate::models::dto::users::user_dto::UserDto;
use crate::models::dto::users::user_info_dto::UserInfoDto;
use crate::models::entities::user::user_role::UserRole;
use crate::models::entities::user::User;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::models::service::password_hash_service::PasswordHashService;
use crate::prelude::*;
use crate::shared_types::SharedPool;
use crate::utils::guard_role::guard_role;
use rocket::serde::json::Json;
use rocket::Route;
use uuid::Uuid;

pub fn create_user_routes() -> Vec<Route> {
    routes![
        get_users,
        get_user_by_id,
        create_user,
        update_user_information,
        update_user_password,
        delete_user,
    ]
}

#[get("/")]
pub async fn get_users(pool: &SharedPool, user: JwtUserPayload) -> Result<Json<Vec<UserDto>>> {
    guard_role(&user.role, UserRole::System)?;

    let inner_pool = pool.inner();

    let records = sqlx::query!(
        r#"
            SELECT Id, Username, Role
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
            role: UserRole::from(record.role),
        })
        .collect();

    Ok(Json(users))
}

#[post("/", data = "<body>")]
pub async fn create_user(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<NewUserDto<'_>>,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;

    let inner_pool = pool.inner();
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

    get_user_by_id(pool, user, uuid.to_string()).await
}

#[get("/<id>")]
pub async fn get_user_by_id(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;

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

#[patch("/<id>", data = "<body>")]
pub async fn update_user_information(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
    body: Json<UserInfoDto<'_>>,
) -> Result<Json<UserDto>> {
    guard_role(&user.role, UserRole::System)?;

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
        body.username,
        role_str
    )
    .execute(inner_pool)
    .await?;

    get_user_by_id(pool, user, id).await
}

#[patch("/<id>/password")]
pub async fn update_user_password(
    pool: &SharedPool,
    user: JwtUserPayload,
    id: String,
) -> Result<Json<UserDto>> {
    todo!()
}

#[delete("/<id>")]
pub async fn delete_user(pool: &SharedPool, user: JwtUserPayload, id: String) -> Result<()> {
    guard_role(&user.role, UserRole::System)?;

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
