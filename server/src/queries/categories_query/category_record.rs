use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct CategoryRecord {
    #[sqlx(rename = "amount")]
    pub amount: Option<i64>,

    #[sqlx(rename = "subcategoryamount")]
    pub subcategory_amount: Option<i64>,

    #[sqlx(rename = "categoryid")]
    pub category_id: String,

    #[sqlx(rename = "categoryname")]
    pub category_name: String,

    #[sqlx(rename = "categorydescription")]
    pub category_description: String,

    #[sqlx(rename = "categoryhexcolor")]
    pub category_hex_color: String,

    #[sqlx(rename = "subcategoryid")]
    pub subcategory_id: Option<String>,

    #[sqlx(rename = "subcategoryname")]
    pub subcategory_name: Option<String>,

    #[sqlx(rename = "subcategorydescription")]
    pub subcategory_description: Option<String>,

    #[sqlx(rename = "subcategoryhexcolor")]
    pub subcategory_hex_color: Option<String>,
}
