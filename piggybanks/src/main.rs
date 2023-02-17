#[macro_use]
extern crate rocket;

mod error;
mod models;
mod prelude;
mod routes;
mod shared_types;
mod utils;

use crate::models::service::jwt_service::JwtService;
use crate::routes::auth::create_auth_routes;

use crate::routes::aggregates::create_aggregate_routes;
use crate::routes::categories::create_category_routes;
use crate::routes::external_accounts::create_external_account_routes;
use crate::routes::importing::create_importing_routes;
use crate::routes::transactions::create_transaction_routes;
use crate::routes::users::create_user_routes;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use sqlx::postgres::PgPoolOptions;
use std::fs;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = dotenv::dotenv();

    let db_connection_string =
        std::env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_connection_string)
        .await
        .expect("Could not create database pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate");

    // Read private key
    let pem_path = std::env::var("PRIVATE_PEM_PATH").expect("PRIVATE_PEM_PATH not set");

    let pem_content = fs::read(pem_path).expect("Failed to read PEM file");

    let pem_string = String::from_utf8(pem_content).expect("Failed to read PEM file");

    let private_key =
        RsaPrivateKey::from_pkcs1_pem(pem_string.as_ref()).expect("Failed to read PEM private key");

    // Read JWT config
    let expire_seconds = std::env::var("JWT_EXPIRE_SECONDS")
        .expect("JWT_EXPIRE_SECONDS not set")
        .parse()
        .expect("JWT_EXPIRE_SECONDS is not an i64");

    let issuer = std::env::var("JWT_ISSUER").expect("JWT_ISSUER not set");

    let jwt_service = JwtService::new(private_key, expire_seconds, issuer);

    let _rocket = rocket::build()
        .manage(pool)
        .manage(jwt_service)
        .mount("/auth", create_auth_routes())
        .mount("/users", create_user_routes())
        .mount("/transactions", create_transaction_routes())
        .mount("/categories", create_category_routes())
        .mount("/external-accounts", create_external_account_routes())
        .mount("/aggregates", create_aggregate_routes())
        .mount("/import", create_importing_routes())
        .launch()
        .await
        .expect("Failed to start rocket");

    Ok(())
}
