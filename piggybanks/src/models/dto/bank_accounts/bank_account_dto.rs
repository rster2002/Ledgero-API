use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountDto {
    pub id: String,
    pub iban: String,
    pub name: String,
    pub description: String,
    pub hex_color: String,
}
