#[macro_use]
extern crate rocket;

/// The shared error type where all the different errors are casted too to create one constant
/// error type.
mod error;

/// Contains all the structs like DTOs and entities.
mod models;

/// Module which allows other files to easily import some universal import.
mod prelude;

/// This module contains all the routes for the API and most of the business logic
/// (yes the business logic is placed with the routing, come at me)
mod routes;

/// Contains alias types for large or commonly used types.
mod shared_types;

/// Shared utility functions used throughout the application.
mod utils;

/// Module for enabling CORS.
mod cors;

/// Contains shared logic that is used throughout the entire application.
pub mod services;

/// Houses certain big queries that need a lot of mapping and config.
pub mod queries;

use crate::routes::auth::create_auth_routes;

use crate::cors::Cors;
use crate::routes::aggregates::create_aggregate_routes;
use crate::routes::categories::create_category_routes;
use crate::routes::external_accounts::create_external_account_routes;
use crate::routes::importing::create_importing_routes;
use crate::routes::transactions::create_transaction_routes;
use crate::routes::users::create_user_routes;
use crate::services::jwt_service::JwtService;
use rocket::http::Status;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use crate::routes::bank_accounts::create_bank_account_routes;

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
        .attach(Cors)
        .manage(pool)
        .manage(jwt_service)
        .mount("/", routes![all_options])
        .mount("/auth", create_auth_routes())
        .mount("/users", create_user_routes())
        .mount("/transactions", create_transaction_routes())
        .mount("/categories", create_category_routes())
        .mount("/bank-accounts", create_bank_account_routes())
        .mount("/external-accounts", create_external_account_routes())
        .mount("/aggregates", create_aggregate_routes())
        .mount("/import", create_importing_routes())
        .launch()
        .await
        .expect("Failed to start rocket");

    Ok(())
}

/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() -> Status {
    Status::Ok
    /* Intentionally left empty */
}
