use std::sync::Arc;
use crate::prelude::*;

pub struct RateLimiter {
    #[cfg(not(test))]
    client: memcache::Client,
}

#[cfg(not(test))]
impl RateLimiter {
    pub fn new(client: memcache::Client) -> Self {
        Self {
            client,
        }
    }

    pub fn limit(
        &self,
        key: impl Into<String>,
        timeout: u32,
    ) -> Result<()> {
        if cfg!(test) {
            return Ok(());
        }

        let key = key.into();
        let existing = self.client.get::<bool>(&key)?;

        if existing.is_some() {
            return Err(Error::RateLimitError);
        }

        self.client.set(&key, true, timeout)?;

        Ok(())
    }
}

#[cfg(test)]
impl RateLimiter {
    pub fn new(client: memcache::Client) -> Self {
        Self {}
    }

    pub fn new_test() -> Self {
        Self {}
    }

    pub fn limit(
        &self,
        key: impl Into<String>,
        timeout: u32,
    ) -> Result<()> {
        Ok(())
    }
}
