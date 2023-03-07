use serde::Serialize;

use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::categories::subcategories::slim_subcategory_dto::SlimSubcategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::entities::transaction::transaction_type::TransactionType;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDto {
    pub id: String,
    pub transaction_type: TransactionType,
    pub follow_number: String,
    pub original_description: String,
    pub description: String,
    pub complete_amount: i64,
    pub amount: i64,
    pub date: String,
    pub bank_account: BankAccountDto,
    pub category: Option<SlimCategoryDto>,
    pub subcategory: Option<SlimSubcategoryDto>,
    pub external_account_name: String,
    pub external_account: Option<ExternalAccountDto>,
}
