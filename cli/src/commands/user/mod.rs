use crate::arguments::cli_commands::UserCommands;
use crate::commands::user::create_user::create_user;
use crate::commands::user::delete_user::delete_user;
use crate::prelude::*;

mod create_user;
mod delete_user;

pub async fn run_user_operation(arguments: UserCommands) -> Result<()> {
    match arguments {
        UserCommands::Create(options) => create_user(options).await,
        UserCommands::Delete(options) => delete_user(options).await,
    }
}
