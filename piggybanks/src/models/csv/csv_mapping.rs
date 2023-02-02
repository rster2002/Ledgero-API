use serde::Deserialize;

/// Used to map a column number to a required field of a transaction. The columns count starts
/// as usual at 0.
#[derive(Debug, Deserialize)]
pub struct CsvMapping {
    pub account_iban: u32,
    pub date: u32,
    pub follow_number: u32,
    pub description: u32,
    pub amount: u32,
    pub external_account_name: u32,
}
