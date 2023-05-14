use rsa::RsaPrivateKey;

/// Options for starting the server.
pub struct StartOptions {
    pub database_url: String,
    pub memcached_url: String,
    pub jwt_signing_key: RsaPrivateKey,
    pub jwt_issuer: String,
    pub jwt_expire_seconds: u32,
    pub max_blob_unconfirmed: u32,
}
