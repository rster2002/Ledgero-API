use serde::Serialize;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::entities::transaction::transaction_type::TransactionType;

#[derive(Debug, Serialize)]
pub struct TransactionDto {
    pub id: String,
    pub transaction_type: TransactionType,
    pub follow_number: String,
    pub original_description: String,
    pub description: String,
    pub complete_amount: i64,
    pub amount: i64,
    pub category: Option<CategoryDto>,
    pub external_account_name: String,
    pub external_account: Option<ExternalAccountDto>,
    pub splits: Vec<TransactionDto>,
}
