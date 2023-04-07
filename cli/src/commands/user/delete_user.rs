use std::env;
use dialoguer::{Confirm, Input};
use dialoguer::theme::ColorfulTheme;
use ledgero_api::services::external_user_service::ExternalUserService;
use crate::arguments::cli_commands::DeleteUserOptions;
use crate::prelude::*;

pub async fn delete_user(options: DeleteUserOptions) -> Result<()> {
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

    let confirm = options.force || Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure?")
        .interact()
        .expect("Failed to start confirmation prompt");

    if !confirm {
        println!("Skipping deletion.");
        return Ok(());
    }

    ExternalUserService::delete_user(
        &db_connection_string,
        &username,
    )
        .await?;

    println!("User deleted!");
    Ok(())
}
