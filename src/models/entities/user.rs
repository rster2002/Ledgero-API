pub mod user_role;

use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;
use crate::shared_types::DbPool;

#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,

    #[sqlx(rename = "PasswordHash")]
    pub password_hash: String,
    pub role: UserRole,
}

impl User {
    pub async fn create(&self, pool: &DbPool) -> Result<()> {
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
