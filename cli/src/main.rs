use clap::Parser;
use crate::arguments::Arguments;
use crate::arguments::cli_operation::CliOperation;
use crate::operations::{manage_user, start};

pub(crate) mod error;
pub(crate) mod arguments;
pub(crate) mod operations;
pub(crate) mod prelude;
pub(crate) mod init;

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    match arguments.operation {
        CliOperation::Start => {
            start().await;
        }
        CliOperation::User(options) => manage_user(options).await
            .unwrap()
    }
}
