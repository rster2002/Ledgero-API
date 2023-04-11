use std::env;
use std::sync::Arc;
use std::time::Duration;

use async_rwlock::RwLock;
use rocket::tokio;

use crate::services::blob_service::BlobService;
use crate::shared::DbPool;

pub fn start_scheduler(blob_service: Arc<RwLock<BlobService>>, pool: Arc<RwLock<DbPool>>) {
    let scheduler_interval = env::var("SCHEDULER_INTERVAL_SECONDS")
        .expect("SCHEDULER_INTERVAL_SECONDS not set")
        .parse()
        .expect("SCHEDULER_INTERVAL_SECONDS is not a u64");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(scheduler_interval));
        info!("Scheduler started");

        loop {
            interval.tick().await;

            let blob_service = blob_service.read().await;
            let pool = pool.read().await;
            let cleanup_result = blob_service.cleanup(&pool).await;

            if let Err(error) = cleanup_result {
                warn!("Failed to run blob cleanup: {:?}", error);
            }
        }
    });
}
