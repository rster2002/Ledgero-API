pub mod cli_operation;

use clap::Parser;
use crate::arguments::cli_operation::CliOperation;

#[derive(Debug, Parser)]
pub struct Arguments {
    #[clap(subcommand)]
    pub operation: CliOperation,
}
