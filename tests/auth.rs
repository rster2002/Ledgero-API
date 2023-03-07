use rocket::serde::json::Json;
use sqlx::PgPool;
use ledgero_api::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use ledgero_api::models::dto::auth::login_user_dto::LoginUserDto;
use ledgero_api::models::dto::auth::register_user_dto::RegisterUserDto;
use ledgero_api::models::entities::user::user_role::UserRole;
use ledgero_api::models::jwt::jwt_user_payload::JwtUserPayload;
use ledgero_api::prelude::*;
use ledgero_api::routes::auth::{register, login, refresh};
use crate::common::*;

mod common;

#[sqlx::test]
async fn user_can_register(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    register(app.pool_state(), Json(RegisterUserDto {
        username: "alice",
        password: "password123",
    }))
        .await?;

    register(app.pool_state(), Json(RegisterUserDto {
        username: "bob",
        password: "password123",
    }))
        .await?;

    Ok(())
}

#[sqlx::test]
async fn user_cannot_use_username_that_is_already_in_use(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    register(app.pool_state(), Json(RegisterUserDto {
        username: "alice",
        password: "password123",
    }))
        .await?;

    let result = register(app.pool_state(), Json(RegisterUserDto {
        username: "alice",
        password: "password123",
    }))
        .await;

    assert!(result.is_err());

    Ok(())
}

#[sqlx::test]
async fn username_has_to_be_at_least_4_characters_long(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    register(app.pool_state(), Json(RegisterUserDto {
        username: "alice",
        password: "password123",
    }))
        .await?;

    let result = register(app.pool_state(), Json(RegisterUserDto {
        username: "abc",
        password: "password123",
    }))
        .await;

    assert!(result.is_err());

    Ok(())
}

#[sqlx::test]
async fn password_has_to_be_at_least_8_characters_long(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    let result = register(app.pool_state(), Json(RegisterUserDto {
        username: "alice",
        password: "abcdefg",
    }))
        .await;

    assert!(result.is_err());

    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn user_can_log_in(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);
    let jwt_service_state = app.jwt_service();

    // Check if user without correct password returns an Err
    let result = login(app.pool_state(), Json(LoginUserDto {
        username: "alice",
        password: "something-else",
    }), app.jwt_service())
        .await;

    assert!(result.is_err());

    // Check if user with the correct password returns an Ok
    let result = login(app.pool_state(), Json(LoginUserDto {
        username: "alice",
        password: "alice",
    }), jwt_service_state)
        .await;

    assert!(result.is_ok());
    let body = result.unwrap().0;

    let (claims, payload) = jwt_service_state.inner().decode_access_token_unchecked::<JwtUserPayload>(body.access_token)
        .unwrap();

    assert_eq!(body.expires, 300);
    assert_eq!(body.token_type, "bearer");

    assert_eq!(payload.uuid, "abc");
    assert_eq!(payload.username, "alice");
    assert_eq!(payload.role, UserRole::User);

    assert_eq!(claims.iss, "tester");

    Ok(())
}

#[sqlx::test(fixtures("users"))]
async fn tokens_can_be_refreshed(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    let login_response = login(app.pool_state(), Json(LoginUserDto {
        username: "alice",
        password: "alice",
    }), app.jwt_service())
        .await
        .unwrap()
        .0;

    let response = refresh(app.pool_state(), Json(JwtRefreshDto {
        access_token: &login_response.access_token,
        refresh_token: &login_response.refresh_token,
    }), app.jwt_service())
        .await;

    assert!(response.is_ok());
    let body = response.unwrap().0;

    assert_ne!(body.access_token, login_response.access_token);
    assert_ne!(body.refresh_token, login_response.refresh_token);

    Ok(())
}
