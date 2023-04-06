use std::env;
use dialoguer::{Input, Password, Select};
use dialoguer::theme::ColorfulTheme;
use crate::prelude::*;
use ledgero_api::services::external_user_service::UserService;
use crate::arguments::cli_operation::CreateUserOptions;

pub async fn create_user(arguments: CreateUserOptions) -> Result<()> {
    let _ = dotenv::dotenv().expect("Failed to load .env file");

    let db_connection_string =
        env::var("DATABASE_URL").expect("Environment variable 'DATABASE_URL' not set");

    let username = arguments.username
        .unwrap_or_else(|| {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Username")
                .interact_text()
                .expect("Failed to prompt for username")
        });

    let password = arguments.password
        .unwrap_or_else(|| {
            Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Password")
                .with_confirmation("Retype password", "Passwords do not match")
                .interact()
                .expect("Failed to prompt initial password")
        });

    let options = ["user", "system"];
    let role = Select::with_theme(&ColorfulTheme::default())
        .items(&options)
        .with_prompt("Role")
        .interact()
        .expect("Failed to prompt role");

    UserService::create_user(
        &db_connection_string,
        &username,
        &password,
        options[role].into()
    )
        .await
        .unwrap();

    println!("User created!");
    Ok(())
}
