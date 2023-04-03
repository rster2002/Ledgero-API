use std::env;

use crate::services::blob_service::BlobService;

pub fn create_blob_service() -> BlobService {
    let max_blob_unconfirmed = env::var("MAX_BLOB_UNCONFIRMED")
        .expect("MAX_BLOB_UNCONFIRMED not set")
        .parse()
        .expect("SCHEDULER_INTERVAL_SECONDS is not a u32");

    BlobService::new(max_blob_unconfirmed).expect("Failed to create blob service")
}
