use std::env;
use dialoguer::{Input, Password, Select};
use dialoguer::theme::ColorfulTheme;
use crate::prelude::*;
use ledgero_api::services::external_user_service::ExternalUserService;
use crate::arguments::cli_commands::CreateUserOptions;

pub async fn create_user(options: CreateUserOptions) -> Result<()> {
    let _ = dotenv::dotenv().expect("Failed to load .env file");

    let db_connection_string =
        env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");

    let username = options.username
        .unwrap_or_else(|| {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Username")
                .interact_text()
                .expect("Failed to prompt for username")
        });

    let password = options.password
        .unwrap_or_else(|| {
            Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Password")
                .with_confirmation("Retype password", "Passwords do not match")
                .interact()
                .expect("Failed to prompt initial password")
        });

    let role_options = ["user", "system"];
    let role = Select::with_theme(&ColorfulTheme::default())
        .items(&role_options)
        .with_prompt("Role")
        .interact()
        .expect("Failed to prompt role");

    ExternalUserService::create_user(
        &db_connection_string,
        &username,
        &password,
        role_options[role].into()
    )
        .await?;

    println!("User created!");
    Ok(())
}
