use serde::Deserialize;

/// Used to map a column number to a required field of a transaction. The columns count starts
/// as usual at 0.
#[derive(Debug, Deserialize)]
pub struct CsvMapping {
    pub account_iban: u32,
    pub date: u32,
    pub date_mapping: DateMapping,
    pub follow_number: u32,
    pub description: u32,
    pub amount: u32,
    pub amount_mapping: AmountMapping,
    pub external_account_name: u32,
}

#[derive(Debug, Deserialize)]
pub enum AmountMapping {
    /// Used when the value of the CSV looks like '129'
    Cents,

    /// Used when the vale of the CSV looks like '1,29' or '1.29'
    Euro,
}

/// Used to configure the datetime mapping for the column.
#[derive(Debug, Deserialize)]
pub struct DateMapping {
    /// Template for formatting the datetime. Used for when the actual value of the date column
    /// cannot be used as a datetime. For example: if the date column has a value '2023-02-03', it
    /// is missing a time, so the template can be set so something like '$ 00:00' where the '$' will
    /// be replaced with the value of the column. Then the result can be parsed using the [format]
    /// option.
    pub template: Option<String>,

    // /// The timezone the date is in. This will be used to convert the datetime to a timezone that
    // /// is the same for every transaction.
    // pub timezone: String,

    /// The format to use when parsing the date. Check the
    /// [chrono documentation](https://docs.rs/chrono/0.4.23/chrono/format/strftime/index.html) for
    /// the possible specifiers that can be used here.
    pub format: String,
}
