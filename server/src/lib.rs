#[macro_use]
extern crate rocket;

use std::env;
use std::sync::Arc;

use async_rwlock::RwLock;
use directories::ProjectDirs;
use hmac::digest::typenum::op;
use rocket::http::Status;
use sqlx::postgres::PgPoolOptions;
use jumpdrive_auth::services::jwt_service::JwtService;

use crate::cors::Cors;
use crate::init::scheduler::start_scheduler;
use crate::init::start_options::StartOptions;
use crate::routes::aggregates::create_aggregate_routes;
use crate::routes::auth::create_auth_routes;
use crate::routes::bank_accounts::create_bank_account_routes;
use crate::routes::blobs::create_blob_routes;
use crate::routes::categories::create_category_routes;
use crate::routes::corrections::create_correction_routes;
use crate::routes::external_accounts::create_external_account_routes;
use crate::routes::importing::create_importing_routes;
use crate::routes::transactions::create_transaction_routes;
use crate::routes::users::create_user_routes;
use crate::services::blob_service::BlobService;
use crate::shared::PROJECT_DIRS;

/// The shared error type where all the different errors are casted too to create one constant
/// error type.
pub mod error;

/// Contains all the structs like DTOs and entities.
pub(crate) mod models;

/// Module which allows other files to easily import some universal import.
pub(crate) mod prelude;

/// This module contains all the routes for the API and most of the business logic
/// (yes the business logic is placed with the routing, come at me)
pub(crate) mod routes;

/// Contains alias types for large or commonly used types.
pub(crate) mod shared;

/// Shared utility functions used throughout the application.
pub(crate) mod utils;

/// Module for enabling CORS.
pub(crate) mod cors;

/// Contains shared logic that is used throughout the entire application.
pub mod services;

/// Houses certain big queries that need a lot of mapping and config.
pub(crate) mod queries;

/// Module for splitting off large chunks of code that needs to be run at startup.
pub mod init;

/// Includes tests for this crate
mod tests;

pub async fn run(options: StartOptions) -> Result<(), rocket::Error> {
    env_logger::init();

    let pool = Arc::new(RwLock::new(
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&options.database_url)
            .await
            .expect("Could not create database pool"),
    ));

    sqlx::migrate!()
        .run(&*(pool.read().await))
        .await
        .expect("Failed to migrate");

    // Configure directories
    let project_dirs =
        ProjectDirs::from("dev", "Jumpdrive", "Ledgero-API").expect("Failed to init directories");

    PROJECT_DIRS
        .set(project_dirs)
        .expect("Failed to share project dirs");

    // Create JWT service
    let jwt_service = JwtService::new(
        options.jwt_signing_key,
        options.jwt_expire_seconds,
        options.jwt_issuer,
        "ledgero-api"
    );

    // Create blob service
    let blob_service = BlobService::new(
        options.max_blob_unconfirmed,
    ).expect("Failed to create blob service");

    // Wrap components in Arc<RwLock> where needed
    let blob_service = Arc::new(RwLock::new(blob_service));

    // Start the scheduler
    start_scheduler(Arc::clone(&blob_service), Arc::clone(&pool));

    info!("Starting server ({})", env!("CARGO_PKG_VERSION"));
    let _ = rocket::build()
        .attach(Cors)
        .manage(pool)
        .manage(jwt_service)
        .manage(blob_service)
        .mount("/", routes![all_options])
        .mount("/auth", create_auth_routes())
        .mount("/users", create_user_routes())
        .mount("/transactions", create_transaction_routes())
        .mount("/corrections", create_correction_routes())
        .mount("/categories", create_category_routes())
        .mount("/bank-accounts", create_bank_account_routes())
        .mount("/external-accounts", create_external_account_routes())
        .mount("/aggregates", create_aggregate_routes())
        .mount("/import", create_importing_routes())
        .mount("/blob", create_blob_routes())
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
