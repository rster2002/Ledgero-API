use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct TransactionRecord {
    pub transaction_id: String,
    pub transaction_type: String,
    pub follow_number: String,
    pub original_description: String,
    pub description: String,
    pub complete_amount: i64,
    pub amount: i64,
    pub date: OffsetDateTime,
    pub related_move_transaction: Option<String>,

    #[sqlx(rename = "bank_account_id?")]
    pub bank_account_id: Option<String>,

    #[sqlx(rename = "bank_account_iban?")]
    pub bank_account_iban: Option<String>,

    #[sqlx(rename = "bank_account_name?")]
    pub bank_account_name: Option<String>,

    #[sqlx(rename = "bank_account_description?")]
    pub bank_account_description: Option<String>,

    #[sqlx(rename = "bank_account_hex_color?")]
    pub bank_account_hex_color: Option<String>,

    #[sqlx(rename = "external_account_name")]
    pub external_account_name: String,

    #[sqlx(rename = "category_id?")]
    pub category_id: Option<String>,

    #[sqlx(rename = "category_name?")]
    pub category_name: Option<String>,

    #[sqlx(rename = "category_description?")]
    pub category_description: Option<String>,

    #[sqlx(rename = "category_hex_color?")]
    pub category_hex_color: Option<String>,

    #[sqlx(rename = "subcategory_id?")]
    pub subcategory_id: Option<String>,

    #[sqlx(rename = "subcategory_name?")]
    pub subcategory_name: Option<String>,

    #[sqlx(rename = "subcategory_description?")]
    pub subcategory_description: Option<String>,

    #[sqlx(rename = "subcategory_hex_color?")]
    pub subcategory_hex_color: Option<String>,

    #[sqlx(rename = "external_account_id?")]
    pub external_account_id: Option<String>,

    #[sqlx(rename = "external_account_entity_name?")]
    pub external_account_entity_name: Option<String>,

    #[sqlx(rename = "external_account_description?")]
    pub external_account_description: Option<String>,

    #[sqlx(rename = "external_account_hex_color?")]
    pub external_account_hex_color: Option<String>,

    #[sqlx(rename = "external_account_default_category_id?")]
    pub external_account_default_category_id: Option<String>,

    #[sqlx(rename = "external_account_default_subcategory_id?")]
    pub external_account_default_subcategory_id: Option<String>,
}
