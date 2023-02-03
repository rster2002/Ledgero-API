use std::fmt::{Display, Formatter};
use rocket::http::Status;
use crate::models::dto::error_dto::{ErrorContent, ErrorDTO};

#[derive(Debug)]
pub enum ImportError {
    /// Indicates that no column could be found for the give mapping. This is usually because the
    /// number given for the column is bigger than the number of columns that exist in the CSV.
    MissingColumn(String),
}

impl ImportError {
    pub fn missing_column(mapping: impl Into<String>) -> ImportError {
        ImportError::MissingColumn(mapping.into())
    }

    pub fn get_status_code(&self) -> u16 {
        let status = match self {
            ImportError::MissingColumn(_) => Status::BadRequest,
        };

        status.code
    }

    pub fn get_body(&self) -> String {
        serde_json::to_string(&ErrorDTO {
            error: ErrorContent {
                code: Status::BadRequest.code,
                reason: "Bad Request".to_string(),
                description: self.to_string(),
            },
        })
            .expect("Failed to serialize error dto")
    }
}

impl Display for ImportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            ImportError::MissingColumn(col) => format!("No column could be found for mapping '{}'", col)
        };

        write!(f, "{}", string)
    }
}
