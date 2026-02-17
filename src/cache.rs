use hyper::body::Bytes;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub body: Bytes,
    pub cached_at: Instant,
}

impl CachedResponse {
    pub fn is_stale(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() > ttl
    }

    pub fn is_servable_if_error(&self, ttl: Duration, stale_if_error: Duration) -> bool {
        self.cached_at.elapsed() < ttl + stale_if_error
    }
}
