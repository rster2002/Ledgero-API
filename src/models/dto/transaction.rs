#[derive(Deserialize)]
pub struct CreateTransaction {
    name: String,
    description: String,
    amount: f64,
}
