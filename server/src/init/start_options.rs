use jumpdrive_auth::services::jwt_service::RsaPrivateKey;

/// Options for starting the server.
pub struct StartOptions {
    pub database_url: String,
    pub memcached_url: String,
    pub jwt_signing_key: RsaPrivateKey,
    pub jwt_issuer: String,
    pub jwt_access_expire_seconds: u32,
    pub jwt_refresh_expire_seconds: u32,
    pub max_blob_unconfirmed: u32,
}
