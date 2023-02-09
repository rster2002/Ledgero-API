pub mod get_transactions;
pub mod splits;

use rocket::Route;

use crate::routes::transactions::get_transactions::*;
use crate::routes::transactions::splits::*;

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
