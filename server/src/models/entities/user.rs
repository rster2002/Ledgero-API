use crate::db_executor;
use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;
use sqlx::{Executor, Postgres};

pub mod user_role;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,

    #[sqlx(rename = "PasswordHash")]
    pub password_hash: String,
    pub role: UserRole,
}

impl User {
    pub async fn create<'r>(&self, pool: db_executor!('r)) -> Result<()> {
        let user_role: &str = self.role.into();

        sqlx::query!(
            r#"
                INSERT INTO Users
                VALUES ($1, $2, $3, $4);
            "#,
            self.id,
            self.username,
            self.password_hash,
            user_role
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
