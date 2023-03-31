#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    ledgero_api::run().await
}
