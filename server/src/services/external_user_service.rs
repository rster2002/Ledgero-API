use rocket::http::hyper::server::conn::Connection;
use crate::db_executor;
use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;
use sqlx::{Executor, Postgres};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
use jumpdrive_auth::services::password_hash_service::PasswordHashService;
use crate::models::entities::user::User;
use crate::utils::single_use_connection::single_use_connection;

/// Primarily used by external crates. The CLI in particular uses this to create users.
pub struct UserService;

impl UserService {
    pub async fn create_user(
        connection_string: &str,
        username: &str,
        password: &str,
        role: UserRole,
    ) -> Result<()> {
        let hash = PasswordHashService::create_new_hash(password);

        let connection = single_use_connection(connection_string)
            .await?;

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: hash,
            role,
        };

        user
            .create(&connection)
            .await?;

        Ok(())
    }
}
