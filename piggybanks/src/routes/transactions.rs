pub mod get_transactions;
pub mod splits;

use std::collections::{BTreeMap, HashMap};
use rocket::http::Status;
use rocket::Route;
use rocket::serde::json::Json;
use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use crate::models::entities::category::Category;
use crate::models::entities::transaction::Transaction;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::routes::transactions::get_transactions::*;
use crate::routes::transactions::splits::*;
use crate::shared_types::SharedPool;

pub fn create_transaction_routes() -> Vec<Route> {
    routes![
        get_all_transactions,
        get_single_transaction,
        change_category_for_transaction,
        get_splits,
        create_split,
        update_split,
        delete_split,
    ]
}

