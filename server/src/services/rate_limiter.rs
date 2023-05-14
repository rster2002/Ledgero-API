use crate::prelude::*;

pub struct RateLimiter {
    memcached_client: memcache::Client,
}

impl RateLimiter {
    pub fn new(client: memcache::Client) -> Self {
        Self {
            memcached_client: client,
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
        let existing = self.memcached_client.get::<bool>(&key)?;

        if existing.is_some() {
            return Err(Error::RateLimitError);
        }

        self.memcached_client.set(&key, true, timeout)?;

        Ok(())
    }
}
