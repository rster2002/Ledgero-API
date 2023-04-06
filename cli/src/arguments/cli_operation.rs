use clap::{Subcommand, Args};

#[derive(Debug, Subcommand)]
pub enum CliOperation {
    /// Starts the API server.
    Start,

    /// Allows you to manage users when the application itself is not online.
    #[clap(subcommand)]
    User(UserOperations),
}

#[derive(Debug, Subcommand)]
pub enum UserOperations {
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
