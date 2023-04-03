use std::sync::Arc;

use async_rwlock::RwLock;
use directories::ProjectDirs;
use once_cell::sync::OnceCell;
use rocket::State;
use sqlx::{Pool, Postgres};

use crate::services::blob_service::BlobService;
use crate::services::jwt_service::JwtService;

pub type DbPool = Pool<Postgres>;

pub type SharedPool = State<Arc<RwLock<DbPool>>>;
pub type SharedJwtService = State<JwtService>;
pub type SharedBlobService = State<Arc<RwLock<BlobService>>>;
pub type DbTransaction<'a> = sqlx::Transaction<'a, Postgres>;

/// Used to create the impl argument type for code that needs an executor. A macro is used here as
/// type aliases that contain the impl keyword are unstable as explained
/// [here](https://github.com/rust-lang/rust/issues/63063).
#[macro_export]
macro_rules! db_executor {
    ($lt:lifetime) => {
        impl Executor<$lt, Database = Postgres>
    }
}

#[macro_export]
macro_rules! db_inner {
    ($name:ident) => {
        &*($name.inner().read().await)
    };
}

pub static PROJECT_DIRS: OnceCell<ProjectDirs> = OnceCell::new();
