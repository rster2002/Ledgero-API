use jumpdrive_auth::services::TotpService;
use rocket::serde::json::Json;
use sqlx::PgPool;
use crate::models::dto::account::enable_mfa_dto::EnableMfaDto;

use crate::models::dto::auth::auth_response_dto::AuthResponseDto;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::dto::auth::login_user_dto::LoginUserDto;
use crate::models::dto::auth::register_user_dto::RegisterUserDto;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_refresh_payload::JwtRefreshPayload;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::routes::auth::get_random_mfa_secret_key;
use crate::routes::auth::login::perform_login;
use crate::routes::auth::refresh_token::refresh;
use crate::routes::auth::registration::register;
use crate::routes::auth::revoke_token::{revoke, revoke_all};
use crate::routes::users::me::enable_mfa_me;
use crate::tests::common::TestApp;

#[sqlx::test]
async fn user_can_register(pool: PgPool) {
    let app = TestApp::new(pool);

    register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "alice",
            password: "password123",
        }),
    )
    .await
    .unwrap();

    register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "bobb",
            password: "password123",
        }),
    )
    .await
    .unwrap();
}

#[sqlx::test]
async fn user_cannot_use_username_that_is_already_in_use(pool: PgPool) {
    let app = TestApp::new(pool);

    register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "alice",
            password: "password123",
        }),
    )
    .await
    .unwrap();

    let result = register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "alice",
            password: "password123",
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn username_has_to_be_at_least_4_characters_long(pool: PgPool) {
    let app = TestApp::new(pool);

    register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "alice",
            password: "password123",
        }),
    )
    .await
    .unwrap();

    let result = register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "abc",
            password: "password123",
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn password_has_to_be_at_least_8_characters_long(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = register(
        app.pool_state(),
        Json(RegisterUserDto {
            username: "alice",
            password: "abcdefg",
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn user_can_log_in(pool: PgPool) {
    let app = TestApp::new(pool);
    let jwt_service_state = app.jwt_service();

    // Check if user without correct password returns an Err
    let result = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "something-else",
            mfa_code: None,
        }),
    )
    .await;

    assert!(result.is_err());

    // Check if user with the correct password returns an Ok
    let result = perform_login(
        app.pool_state(),
        jwt_service_state,
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
    .await;

    assert!(result.is_ok());
    let body = result.unwrap()
        .0
        .unwrap_jwt_access_token();

    let (claims, payload) = jwt_service_state
        .inner()
        .decode_access_token_unchecked::<JwtUserPayload>(body.access_token)
        .unwrap();

    assert_eq!(body.expires, 300);
    assert_eq!(body.token_type, "bearer");

    assert_eq!(payload.uuid, "abc");
    assert_eq!(payload.username, "alice");
    assert_eq!(payload.role, UserRole::User);

    assert_eq!(claims.iss, "tester");
}

#[sqlx::test(fixtures("users"))]
async fn tokens_can_be_refreshed(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
    .await
    .unwrap()
    .0
    .unwrap_jwt_access_token();

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let response = refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response.access_token,
            refresh_token: &login_response.refresh_token,
        }),
    )
    .await;

    assert!(response.is_ok());
    let body = response.unwrap()
        .0
        .unwrap_jwt_access_token();

    assert_ne!(body.access_token, login_response.access_token);
    assert_ne!(body.refresh_token, login_response.refresh_token);

    // Refreshing a refresh token should not update it's expire time
    let jwt_service = app.jwt_service();

    let old_refresh_claims = jwt_service
        .decode_refresh_token_unchecked::<JwtRefreshPayload>(body.refresh_token)
        .unwrap()
        .0;

    let new_refresh_claims = jwt_service
        .decode_refresh_token_unchecked::<JwtRefreshPayload>(login_response.refresh_token)
        .unwrap()
        .0;

    assert!(new_refresh_claims.exp - old_refresh_claims.exp < 100);
}

#[sqlx::test(fixtures("users"))]
async fn tokens_cannot_be_refreshed_multiple_times(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
    .await
    .unwrap()
    .0
    .unwrap_jwt_access_token();

    refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response.access_token,
            refresh_token: &login_response.refresh_token,
        }),
    )
    .await
    .unwrap();

    let result = refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response.access_token,
            refresh_token: &login_response.refresh_token,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn tokens_can_be_revoked(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
    .await
    .unwrap()
    .0
    .unwrap_jwt_access_token();

    revoke(
        app.pool_state(),
        app.jwt_service(),
        Json(RevokeDto {
            refresh_token: login_response.refresh_token.to_string(),
        }),
    )
    .await
    .unwrap();

    let result = refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response.access_token,
            refresh_token: &login_response.refresh_token,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn user_can_be_logged_out_everywhere(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_response_1 = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
        .await
        .unwrap()
        .0
        .unwrap_jwt_access_token();

    let login_response_2 = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        }),
    )
        .await
        .unwrap()
        .0
        .unwrap_jwt_access_token();

    let response = revoke_all(
        app.pool_state(),
        app.alice(),
    )
        .await;

    assert!(response.is_ok());

    let refresh_1_result = refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response_1.access_token,
            refresh_token: &login_response_1.refresh_token,
        })
    )
        .await;

    let refresh_2_result = refresh(
        app.pool_state(),
        app.jwt_service(),
        Json(JwtRefreshDto {
            access_token: &login_response_2.access_token,
            refresh_token: &login_response_2.refresh_token,
        })
    )
        .await;

    assert!(refresh_1_result.is_err());
    assert!(refresh_2_result.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn user_with_mfa_can_log_in_using_correct_code(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: None,
        }),
    )
        .await
        .unwrap()
        .0;

    assert!(matches!(login_response, AuthResponseDto::TwoFAChallenge));

    let current_code = TotpService::test_generate_current_code("MU3EEY32LJTXIMKHKQ3TAR2MIJVUIVKM")
        .unwrap();

    let _ = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: Some(&current_code),
        }),
    )
        .await
        .unwrap()
        .0
        .unwrap_jwt_access_token();
}

#[sqlx::test(fixtures("users"))]
async fn user_with_mfa_cannot_log_in_using_incorrect_code(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_result = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: Some("123456"),
        }),
    )
        .await;

    assert!(login_result.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn user_with_mfa_can_log_in_using_backup_code(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_result = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: Some("abcd1234"),
        }),
    )
        .await;

    assert!(login_result.is_ok());
}

#[sqlx::test(fixtures("users"))]
async fn cannot_use_the_same_mfa_backup_code_multiple_times(pool: PgPool) {
    let app = TestApp::new(pool);

    let login_result_1 = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: Some("abcd1234"),
        }),
    )
        .await;

    assert!(login_result_1.is_ok());

    let login_result_2 = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "charley",
            password: "alice",
            mfa_code: Some("abcd1234"),
        }),
    )
        .await;

    assert!(login_result_2.is_err());
}

#[sqlx::test(fixtures("users"))]
async fn mfa_can_be_enabled(pool: PgPool) {
    let app = TestApp::new(pool);

    let secret_key = get_random_mfa_secret_key()
        .await
        .unwrap()
        .0
        .secret_key;

    let code = TotpService::test_generate_current_code(&secret_key)
        .unwrap();

    enable_mfa_me(
        app.pool_state(),
        app.alice(),
        Json(EnableMfaDto {
            secret_key: secret_key.to_string(),
            code,
        })
    )
        .await
        .unwrap();

    let initial_login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: None,
        })
    )
        .await
        .unwrap()
        .0;

    assert!(matches!(initial_login_response, AuthResponseDto::TwoFAChallenge));

    let login_code = TotpService::test_generate_current_code(&secret_key)
        .unwrap();

    let mfa_login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: Some(&login_code),
        })
    )
        .await
        .unwrap()
        .0;

    assert!(matches!(mfa_login_response, AuthResponseDto::JwtAccessToken(_)));
}

#[sqlx::test(fixtures("users"))]
async fn mfa_new_backup_codes_can_be_used(pool: PgPool) {
    let app = TestApp::new(pool);

    let secret_key = get_random_mfa_secret_key()
        .await
        .unwrap()
        .0
        .secret_key;

    let code = TotpService::test_generate_current_code(&secret_key)
        .unwrap();

    let backup_codes = enable_mfa_me(
        app.pool_state(),
        app.alice(),
        Json(EnableMfaDto {
            secret_key: secret_key.to_string(),
            code,
        })
    )
        .await
        .unwrap()
        .0
        .backup_codes;

    let mfa_login_response = perform_login(
        app.pool_state(),
        app.jwt_service(),
        app.rate_limiter(),
        Json(LoginUserDto {
            username: "alice",
            password: "alice",
            mfa_code: Some(&backup_codes[0]),
        })
    )
        .await
        .unwrap()
        .0;

    assert!(matches!(mfa_login_response, AuthResponseDto::JwtAccessToken(_)));
}
