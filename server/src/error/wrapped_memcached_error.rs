use std::fmt::Debug;
use memcache::MemcacheError;
use rocket::http::Status;
use crate::error::error_dto_trait::ToErrorDto;

#[derive(Debug)]
pub struct WrappedMemcachedError {
    inner: MemcacheError,
}

impl WrappedMemcachedError {
    pub fn new(value: MemcacheError) -> Self {
        Self {
            inner: value,
        }
    }
}

impl ToErrorDto for WrappedMemcachedError {
    fn get_status_code(&self) -> Status {
        Status::InternalServerError
    }

    fn get_description(&self) -> String {
        let slice = match self.inner {
            MemcacheError::BadURL(_) => "Failed to connect to Memcached due to bad URL",
            MemcacheError::IOError(_) => "Failed to connect to Memcached due an IO error",
            MemcacheError::ClientError(_) => "Memcached client error",
            MemcacheError::ServerError(_) => "Memcached server error",
            MemcacheError::CommandError(_) => "Memcached command error",
            MemcacheError::ParseError(_) => "Memcached parse error",
            MemcacheError::PoolError(_) => "Memcached pool error",
        };

        slice.to_string()
    }
}
