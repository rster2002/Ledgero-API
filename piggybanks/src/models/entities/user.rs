use entity_macro::{table_name, Entity};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Entity)]
#[table_name("Users")]
#[sqlx(rename_all = "PascalCase")]
pub struct User {
    pub id: String,
    pub username: String,

    #[sqlx(rename = "PasswordHash")]
    pub password_hash: String,
}

impl User {
    pub fn new(username: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username: username.into(),
            password_hash: password_hash.into(),
        }
    }
}
