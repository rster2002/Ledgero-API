use crate::models::dto::error_dto::{ErrorContent, ErrorDTO};
use rocket::http::Status;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ImportError {
    /// Indicates that no column could be found for the give mapping. This is usually because the
    /// number given for the column is bigger than the number of columns that exist in the CSV.
    MissingColumn(String),

    /// Indicates that there are no rows to import. This is mainly used for the dry run route as the
    /// normal import routes will iterate over the records so manual checking is not required.
    NoRows,
}

impl ImportError {
    pub fn missing_column(mapping: impl Into<String>) -> ImportError {
        ImportError::MissingColumn(mapping.into())
    }

    pub fn get_status_code(&self) -> u16 {
        let status = match self {
            ImportError::MissingColumn(_)
            | ImportError::NoRows => Status::BadRequest,
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
            ImportError::MissingColumn(col) => {
                format!("No column could be found for mapping '{}'", col)
            },
            ImportError::NoRows => "The CSV did not contain any rows".to_string(),
        };

        write!(f, "{}", string)
    }
}
