use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::cache::CachedResponse;
use hyper::body::Bytes;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Option<CachedResponse>;
    async fn set(&self, key: String, value: CachedResponse);
    async fn size(&self) -> usize;
}

pub struct MemoryStorage {
    cache: RwLock<HashMap<String, CachedResponse>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Option<CachedResponse> {
        self.cache.read().await.get(key).cloned()
    }

    async fn set(&self, key: String, value: CachedResponse) {
        self.cache.write().await.insert(key, value);
    }

    async fn size(&self) -> usize {
        self.cache.read().await.len()
    }
}

pub struct RedisStorage {
    client: redis::aio::ConnectionManager,
}

impl RedisStorage {
    pub async fn new(url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let connection_manager = redis::aio::ConnectionManager::new(client).await?;
        Ok(Self {
            client: connection_manager,
        })
    }
}

#[async_trait]
impl Storage for RedisStorage {
    async fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut conn = self.client.clone();

        let result: Result<(Vec<u8>, u64), redis::RedisError> = redis::pipe()
            .get(format!("{key}:body"))
            .get(format!("{key}:cached_at"))
            .query_async(&mut conn)
            .await;

        match result {
            Ok((body, cached_at_nanos)) => {
                let elapsed = Duration::from_nanos(cached_at_nanos);
                let cached_at = Instant::now() - elapsed;
                Some(CachedResponse {
                    body: Bytes::from(body),
                    cached_at,
                })
            }
            Err(_) => None,
        }
    }

    async fn set(&self, key: String, value: CachedResponse) {
        let mut conn = self.client.clone();
        let elapsed = value.cached_at.elapsed().as_nanos() as u64;

        let _: Result<(), redis::RedisError> = redis::pipe()
            .set(format!("{key}:body"), value.body.to_vec())
            .set(format!("{key}:cached_at"), elapsed)
            .query_async(&mut conn)
            .await;
    }

    async fn size(&self) -> usize {
        0
    }
}

pub type Cache = Arc<dyn Storage>;
