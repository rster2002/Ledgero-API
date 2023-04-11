pub mod cli_commands;

use clap::Parser;
use crate::arguments::cli_commands::CliCommands;

#[derive(Debug, Parser)]
pub struct Arguments {
    #[clap(subcommand)]
    pub command: CliCommands,
}
