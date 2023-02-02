use sqlx::FromRow;
use entity_macro::{Entity, table_name};

#[derive(Debug, FromRow, Entity)]
#[table_name("Categories")]
#[sqlx(rename_all = "PascalCase")]
pub struct Category {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}
