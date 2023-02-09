use crate::models::csv::csv_mapping::CsvMapping;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ImportCsvDto {
    pub mappings: CsvMapping,
    pub csv: String,
}
