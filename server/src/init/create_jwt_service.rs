use std::{env, fs};

use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::RsaPrivateKey;

use crate::services::jwt_service::JwtService;

pub fn create_jwt_service() -> JwtService {
    let pem_path = env::var("PRIVATE_PEM_PATH").expect("PRIVATE_PEM_PATH not set");

    let pem_content = fs::read(pem_path).expect("Failed to read PEM file");

    let pem_string = String::from_utf8(pem_content).expect("Failed to read PEM file");

    let private_key =
        RsaPrivateKey::from_pkcs1_pem(pem_string.as_ref()).expect("Failed to read PEM private key");

    let expire_seconds = std::env::var("JWT_EXPIRE_SECONDS")
        .expect("JWT_EXPIRE_SECONDS not set")
        .parse()
        .expect("JWT_EXPIRE_SECONDS is not an i64");

    let issuer = env::var("JWT_ISSUER").expect("JWT_ISSUER not set");

    JwtService::new(private_key, expire_seconds, issuer)
}
