use clap::{Subcommand, Args};

#[derive(Debug, Subcommand)]
pub enum CliCommands {
    /// Starts the API server.
    Start,

    /// Allows you to manage users when the application itself is not online.
    #[clap(subcommand)]
    User(UserCommands),
}

#[derive(Debug, Subcommand)]
pub enum UserCommands {
    Create(CreateUserOptions),
    Delete(DeleteUserOptions),
}

#[derive(Debug, Args)]
pub struct CreateUserOptions {
    #[arg(short, long)]
    pub username: Option<String>,

    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Debug, Args)]
pub struct DeleteUserOptions {
    #[arg(short, long)]
    pub username: Option<String>,

    #[arg(long)]
    pub force: bool,
}
