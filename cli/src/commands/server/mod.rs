use crate::commands::server::create_start_options::create_start_options;

mod create_start_options;

pub async fn start_server() {
    let _ = dotenv::dotenv();

    let start_options = create_start_options();

    ledgero_api::run(start_options)
        .await
        .unwrap();
}
