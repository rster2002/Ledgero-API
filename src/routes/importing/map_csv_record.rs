use chrono::{DateTime, Utc};
use csv::StringRecord;

use crate::error::import_error::ImportError;
use crate::models::csv::csv_mapping::{AmountMapping, CsvMapping, DateMapping};
use crate::prelude::*;

#[derive(Debug)]
pub struct MappedCsvRecord {
    pub account_iban: String,
    pub date: DateTime<Utc>,
    pub follow_number: String,
    pub description: String,
    pub amount: i64,
    pub external_account_name: String,
}

pub fn map_csv_record(record: StringRecord, mapping: &CsvMapping) -> Result<MappedCsvRecord> {
    let follow_number = record
        .get(mapping.follow_number as usize)
        .ok_or(ImportError::missing_column("follow_number"))?
        .to_string();

    let description = record
        .get(mapping.description as usize)
        .ok_or(ImportError::missing_column("description"))?
        .to_string();

    let temp_amount = record
        .get(mapping.amount as usize)
        .ok_or(ImportError::missing_column("amount"))?
        .replace('+', "")
        .replace(',', ".")
        .parse::<f64>()?;

    let amount: i64 = match mapping.amount_mapping {
        AmountMapping::Cents => temp_amount as i64,
        AmountMapping::Euro => (temp_amount * 100_f64) as i64,
    };

    let date_string = record
        .get(mapping.date as usize)
        .ok_or(ImportError::missing_column("date"))?
        .to_string();

    let date = map_datetime(&date_string, &mapping.date_mapping)?;

    let account_iban = record
        .get(mapping.account_iban as usize)
        .ok_or(ImportError::missing_column("iban"))?
        .to_string();

    let external_account_name = record
        .get(mapping.external_account_name as usize)
        .ok_or("Column for external_account_name does not exist")?
        .to_string();

    Ok(MappedCsvRecord {
        account_iban,
        date,
        follow_number,
        description,
        amount,
        external_account_name,
    })
}

fn map_datetime(col_value: &String, date_mapping: &DateMapping) -> Result<DateTime<Utc>> {
    let mut working_value = col_value.to_string();

    if let Some(template) = &date_mapping.template {
        working_value = template.replace('$', &working_value);
    }

    let datetime = DateTime::parse_from_str(&working_value, &date_mapping.format)?;

    Ok(DateTime::from(datetime))
}
