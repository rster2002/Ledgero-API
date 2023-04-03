mod error;

#[tokio::main]
async fn main() {
    ledgero_api::run()
        .await
        .unwrap();
}
