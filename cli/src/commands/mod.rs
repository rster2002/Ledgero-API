mod user;
mod server;

use crate::arguments::cli_commands::{CliCommands};
use crate::commands::server::start_server;
use crate::commands::user::run_user_operation;

pub async fn run_command(operation: CliCommands) {
    match operation {
        CliCommands::Start => {
            start_server().await;
        }
        CliCommands::User(options) => run_user_operation(options).await
            .unwrap()
    }
}
