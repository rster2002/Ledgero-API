use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct CategoryRecord {
    pub amount: Option<i64>,
    pub subcategory_amount: Option<i64>,
    pub category_id: String,
    pub category_name: String,
    pub category_description: String,
    pub category_hex_color: String,
    pub subcategory_id: Option<String>,
    pub subcategory_name: Option<String>,
    pub subcategory_description: Option<String>,
    pub subcategory_hex_color: Option<String>,
}
