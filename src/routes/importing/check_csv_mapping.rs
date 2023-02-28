use std::io::Cursor;

use crate::error::import_error::ImportError;
use crate::models::dto::importing::check_csv_mapping_dto::CheckCsvMappingDto;
use crate::models::dto::importing::import_csv_dto::ImportCsvDto;
use rocket::serde::json::Json;

use crate::prelude::*;
use crate::routes::importing::map_csv_record::map_csv_record;

#[post("/csv/check-mapping", data = "<body>")]
pub async fn check_csv_mapping(body: Json<ImportCsvDto>) -> Result<Json<CheckCsvMappingDto>> {
    let body = body.0;

    let mut reader = csv::Reader::from_reader(Cursor::new(body.csv));
    let read_record = reader.records().next();

    let Some(record) = read_record else {
        return Err(ImportError::NoRows.into());
    };

    let mapped_record = map_csv_record(record?, &body.mappings)?;

    Ok(Json(CheckCsvMappingDto {
        account_iban: mapped_record.account_iban,
        date: mapped_record.date.to_string(),
        follow_number: mapped_record.follow_number,
        description: mapped_record.description,
        amount: mapped_record.amount,
        external_account_name: mapped_record.external_account_name,
    }))
}
