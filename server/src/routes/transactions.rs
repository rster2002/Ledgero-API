use rocket::Route;

use crate::routes::transactions::splits::*;
use crate::routes::transactions::transaction_management::*;

pub mod splits;
pub mod transaction_management;

pub fn create_transaction_routes() -> Vec<Route> {
    routes![
        get_all_transactions,
        get_single_transaction,
        change_category_for_transaction,
        update_transaction_details,
        update_transaction,
        get_splits,
        create_split,
        update_split,
        delete_split,
    ]
}
