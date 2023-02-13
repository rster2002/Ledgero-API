use crate::models::csv::csv_mapping::CsvMapping;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImportCsvDto {
    pub mappings: CsvMapping,
    pub filename: String,
    pub csv: String,
}
