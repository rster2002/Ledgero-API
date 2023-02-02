use serde::Deserialize;
use crate::models::csv::csv_mapping::CsvMapping;

#[derive(Debug, Deserialize)]
pub struct ImportCsvDto {
    mappings: CsvMapping,
    csv: String,
}
