use sqlx::FromRow;
use sqlx::types::time::OffsetDateTime;

#[derive(FromRow)]
pub struct TransactionRecord {
    #[sqlx(rename = "transactionid")]
    pub transaction_id: String,

    #[sqlx(rename = "transactiontype")]
    pub transaction_type: String,

    #[sqlx(rename = "follownumber")]
    pub follow_number: String,

    #[sqlx(rename = "originaldescription")]
    pub original_description: String,

    #[sqlx(rename = "description")]
    pub description: String,

    #[sqlx(rename = "completeamount")]
    pub complete_amount: i64,

    #[sqlx(rename = "amount")]
    pub amount: i64,

    #[sqlx(rename = "date")]
    pub date: OffsetDateTime,

    #[sqlx(rename = "bankaccountid")]
    pub bank_account_id: String,

    #[sqlx(rename = "bankaccountiban")]
    pub bank_account_iban: String,

    #[sqlx(rename = "bankaccountname")]
    pub bank_account_name: String,

    #[sqlx(rename = "bankaccountdescription")]
    pub bank_account_description: String,

    #[sqlx(rename = "bankaccounthexcolor")]
    pub bank_account_hex_color: String,

    #[sqlx(rename = "externalaccountname")]
    pub external_account_name: String,

    #[sqlx(rename = "CategoryId?")]
    pub category_id: Option<String>,

    #[sqlx(rename = "CategoryName?")]
    pub category_name: Option<String>,

    #[sqlx(rename = "CategoryDescription?")]
    pub category_description: Option<String>,

    #[sqlx(rename = "CategoryHexColor?")]
    pub category_hex_color: Option<String>,

    #[sqlx(rename = "SubcategoryId?")]
    pub subcategory_id: Option<String>,

    #[sqlx(rename = "SubcategoryName?")]
    pub subcategory_name: Option<String>,

    #[sqlx(rename = "SubcategoryDescription?")]
    pub subcategory_description: Option<String>,

    #[sqlx(rename = "SubcategoryHexColor?")]
    pub subcategory_hex_color: Option<String>,

    #[sqlx(rename = "ExternalAccountId?")]
    pub external_account_associated_name: Option<String>,

    #[sqlx(rename = "ExternalAccountEntityName?")]
    pub external_account_entity_name: Option<String>,

    #[sqlx(rename = "ExternalAccountDescription?")]
    pub external_account_description: Option<String>,

    #[sqlx(rename = "ExternalAccountHexColor?")]
    pub external_account_hex_color: Option<String>,

    #[sqlx(rename = "ExternalAccounDefaultCategoryId?")]
    pub external_account_default_category_id: Option<String>,
}
