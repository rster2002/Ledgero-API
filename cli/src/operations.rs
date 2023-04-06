pub mod create_user;

use crate::arguments::cli_operation::UserOperations;
use crate::init::create_start_options;
use crate::operations::create_user::create_user;
use crate::prelude::*;

pub async fn start() {
    let _ = dotenv::dotenv();

    let start_options = create_start_options();

    ledgero_api::run(start_options)
        .await
        .unwrap();
}

pub async fn manage_user(arguments: UserOperations) -> Result<()> {
    match arguments {
        UserOperations::Create(options) => create_user(options).await,
        UserOperations::Delete(_) => {
            todo!()
        }
    }
}
