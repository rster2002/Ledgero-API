use rocket::http::Status;
use sqlx::Error;

use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub struct WrappedSqlxError {
    inner: Error,
}

impl WrappedSqlxError {
    pub fn new(error: Error) -> Self {
        Self { inner: error }
    }

    pub fn get_constraint(&self) -> Option<&str> {
        let Error::Database(error) = &self.inner else {
            return None;
        };

        error.constraint()
    }
}

impl ToErrorDto for WrappedSqlxError {
    fn get_status_code(&self) -> Status {
        match self.inner {
            Error::RowNotFound => Status::NotFound,
            _ => Status::InternalServerError,
        }
    }

    #[cfg(debug_assertions)]
    fn get_description(&self) -> String {
        self.inner.to_string()
    }

    #[cfg(not(debug_assertions))]
    fn get_description(&self) -> String {
        let message: &str = match &self.inner {
            Error::Configuration(_) => "Invalid database configuration",
            Error::Database(error) => {
                if let Some(constraint) = error.constraint() {
                    return format!("Failed constraint: {}", constraint);
                }

                "Database error"
            }
            Error::Io(_) => "IO error",
            Error::Tls(_) => "TLS error",
            Error::Protocol(_) => "Protocol error",
            Error::RowNotFound => "Entity could not be found",
            Error::TypeNotFound { .. } => "Type was not found",
            Error::ColumnIndexOutOfBounds { .. } => "Column index out of bounds",
            Error::ColumnNotFound(_) => "Column could not be found",
            Error::ColumnDecode { .. } => "Failed to decode column",
            Error::Decode(_) => "Failed to decode",
            Error::PoolTimedOut => "Database pool timed out",
            Error::PoolClosed => "Database pool closed",
            Error::WorkerCrashed => "The database worker crashed",
            Error::Migrate(_) => "Failed to migrate database",
            _ => "Unknown error",
        };

        message.to_string()
    }
}
