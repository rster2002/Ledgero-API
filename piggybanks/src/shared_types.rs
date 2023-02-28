use rocket::State;
use sqlx::{Executor, Pool, Postgres};
use crate::services::jwt_service::JwtService;

pub type DbPool = Pool<Postgres>;

pub type SharedPool = State<DbPool>;
pub type SharedJwtService = State<JwtService>;
pub type DbTransaction<'a> = sqlx::Transaction<'a, Postgres>;

/// Used to create the impl argument type for code that needs an executor
#[macro_export]
macro_rules! db_executor {
    ($lt:lifetime) => {
        impl Executor<$lt, Database = Postgres>
    }
}
