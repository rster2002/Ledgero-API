use std::{env, fs};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;
use ledgero_api::init::start_options::StartOptions;

pub fn create_start_options() -> StartOptions {
    let db_connection_string =
        env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");

    let memcached_connection_string =
        env::var("MEMCACHED_URL").expect("Environment variable 'MEMCACHED_URL' not set");

    // JWT options
    let issuer = env::var("JWT_ISSUER").expect("JWT_ISSUER not set");

    let expire_seconds = std::env::var("JWT_EXPIRE_SECONDS")
        .expect("JWT_EXPIRE_SECONDS not set")
        .parse()
        .expect("JWT_EXPIRE_SECONDS is not an i64");

    let pem_path = env::var("PRIVATE_PEM_PATH").expect("PRIVATE_PEM_PATH not set");

    let pem_content = fs::read(pem_path).expect("Failed to read PEM file");

    let pem_string = String::from_utf8(pem_content).expect("Failed to read PEM file");

    let private_key =
        RsaPrivateKey::from_pkcs1_pem(pem_string.as_ref()).expect("Failed to read PEM private key");

    // Blob options
    let max_blob_unconfirmed = env::var("MAX_BLOB_UNCONFIRMED")
        .expect("MAX_BLOB_UNCONFIRMED not set")
        .parse()
        .expect("MAX_BLOB_UNCONFIRMED is not a u32");

    StartOptions {
        database_url: db_connection_string,
        memcached_url: memcached_connection_string,
        jwt_issuer: issuer,
        jwt_expire_seconds: expire_seconds,
        jwt_signing_key: private_key,
        max_blob_unconfirmed,
    }
}
