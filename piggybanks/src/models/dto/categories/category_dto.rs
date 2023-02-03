use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
    pub amount: Option<i64>,
}
