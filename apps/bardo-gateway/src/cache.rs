//! Async response cache backed by moka with LRU eviction and TTL.

use std::time::Duration;

use moka::future::Cache;

use crate::state::CachedResponse;

/// Async LRU + TTL response cache.
///
/// Wraps a [`moka::future::Cache`] keyed by blake3 request hashes.
/// Entries are evicted when the cache exceeds `max_entries` (LRU) or when
/// an entry's TTL expires, whichever comes first.
pub struct ResponseCache {
    inner: Cache<[u8; 32], CachedResponse>,
}

impl ResponseCache {
    /// Create a new cache with the given capacity and TTL.
    pub fn new(max_entries: u64, ttl_seconds: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();
        Self { inner }
    }

    /// Compute a blake3 hash of the request body for cache lookup.
    pub fn request_hash(body: &[u8]) -> [u8; 32] {
        blake3::hash(body).into()
    }

    /// Try to get a cached response by its hash.
    pub async fn get(&self, hash: &[u8; 32]) -> Option<CachedResponse> {
        self.inner.get(hash).await
    }

    /// Store a response in the cache.
    pub async fn put(&self, hash: [u8; 32], response: CachedResponse) {
        self.inner.insert(hash, response).await;
    }

    /// Number of entries currently in the cache.
    pub fn len(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.entry_count() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    fn make_cached(body: &str) -> CachedResponse {
        CachedResponse {
            body: Bytes::from(body.to_string()),
            content_type: "application/json".into(),
            cost_usd: 0.01,
            model: "test-model".into(),
            cached_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn put_and_get() {
        let cache = ResponseCache::new(100, 3600);
        let hash = ResponseCache::request_hash(b"test body");
        cache.put(hash, make_cached("response")).await;
        let got = cache.get(&hash).await;
        assert!(got.is_some());
        assert_eq!(got.map(|r| r.model), Some("test-model".into()));
    }

    #[tokio::test]
    async fn miss_on_unknown_hash() {
        let cache = ResponseCache::new(100, 3600);
        let hash = ResponseCache::request_hash(b"never inserted");
        assert!(cache.get(&hash).await.is_none());
    }

    #[tokio::test]
    async fn len_tracks_entries() {
        let cache = ResponseCache::new(100, 3600);
        assert!(cache.is_empty());
        let h1 = ResponseCache::request_hash(b"a");
        let h2 = ResponseCache::request_hash(b"b");
        cache.put(h1, make_cached("r1")).await;
        cache.put(h2, make_cached("r2")).await;
        // moka's entry_count() is eventually consistent; run pending tasks
        cache.inner.run_pending_tasks().await;
        assert_eq!(cache.len(), 2);
    }
}
