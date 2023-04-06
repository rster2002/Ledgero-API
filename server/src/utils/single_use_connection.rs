use sqlx::postgres::PgPoolOptions;

use crate::prelude::*;
use crate::shared::DbPool;

pub async fn single_use_connection(connection_string: &str) -> Result<DbPool> {
    Ok(PgPoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await?)
}
