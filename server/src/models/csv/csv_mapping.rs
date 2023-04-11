use serde::Deserialize;

/// Used to map a column number to a required field of a transaction. The columns count starts
/// as usual at 0.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvMapping {
    pub date_mapping: DateMapping,
    pub amount_mapping: AmountMapping,
    pub ordering: CsvImportOrdering,

    pub account_iban: u32,
    pub date: u32,
    pub follow_number: u32,
    pub description: u32,
    pub amount: u32,
    pub external_account_name: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AmountMapping {
    /// Used when the value of the CSV looks like '129'
    Cents,

    /// Used when the vale of the CSV looks like '1,29' or '1.29'
    Euro,
}

/// Used to configure the datetime mapping for the column.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateMapping {
    /// Template for formatting the datetime. Used for when the actual value of the date column
    /// cannot be used as a datetime. For example: if the date column has a value '2023-02-03', it
    /// is missing a time, so the template can be set so something like '$ 00:00' where the '$' will
    /// be replaced with the value of the column. Then the result can be parsed using the [format]
    /// option.
    pub template: Option<String>,

    /// The format to use when parsing the date. Check the
    /// [chrono documentation](https://docs.rs/chrono/0.4.23/chrono/format/strftime/index.html) for
    /// the possible specifiers that can be used here.
    pub format: String,
}

/// Used for populating the [Transaction::order_indicator] field which helps with keeping the
/// correct order of the transactions.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CsvImportOrdering {
    /// Indicates that the newest transaction is the first row in the CSV.
    NewestFirst,

    /// Indicates that the newest transaction is the last row in the CSV.
    NewestLast,
}
