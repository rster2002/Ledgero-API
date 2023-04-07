use clap::Parser;
use crate::arguments::Arguments;
use crate::arguments::cli_commands::CliCommands;
use crate::commands::{run_command};

pub(crate) mod error;
pub(crate) mod arguments;
pub(crate) mod commands;
pub(crate) mod prelude;

#[tokio::main]
async fn main() {
    let arguments = Arguments::parse();

    run_command(arguments.command)
        .await;
}
