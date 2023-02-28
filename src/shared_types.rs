use rocket::State;
use sqlx::{Executor, Pool, Postgres};
use crate::services::jwt_service::JwtService;

pub type DbPool = Pool<Postgres>;

pub type SharedPool = State<DbPool>;
pub type SharedJwtService = State<JwtService>;
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
