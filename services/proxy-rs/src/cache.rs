use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct CacheStore {
    inner: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
    max_entries: usize,
}

impl CacheStore {
    pub fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries,
        }
    }

    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut guard = self.inner.write().await;
        match guard.get(key) {
            Some(entry) if !entry.is_expired(self.ttl) => Some(entry.response.clone()),
            Some(_) => {
                guard.remove(key);
                None
            }
            None => None,
        }
    }

    pub async fn insert(&self, key: String, response: CachedResponse) {
        let mut guard = self.inner.write().await;
        guard.insert(
            key,
            CacheEntry {
                response,
                created_at: Instant::now(),
            },
        );
        Self::enforce_capacity(&mut guard, self.max_entries);
    }

    pub async fn purge(&self, selector: CachePurgeSelector) -> usize {
        let mut guard = self.inner.write().await;
        let before = guard.len();
        match selector {
            CachePurgeSelector::All => guard.clear(),
            CachePurgeSelector::Prefix(prefix) => {
                guard.retain(|key, _| !key.starts_with(&prefix));
            }
        }
        before.saturating_sub(guard.len())
    }

    pub async fn stats(&self) -> CacheStats {
        let guard = self.inner.read().await;
        let mut expired_entries = 0;
        let mut oldest_age_seconds = 0;

        for entry in guard.values() {
            let age_seconds = entry.created_at.elapsed().as_secs();
            oldest_age_seconds = oldest_age_seconds.max(age_seconds);
            if entry.is_expired(self.ttl) {
                expired_entries += 1;
            }
        }

        CacheStats {
            entries: guard.len(),
            expired_entries,
            max_entries: self.max_entries,
            ttl_seconds: self.ttl.as_secs(),
            oldest_age_seconds,
        }
    }

    pub async fn sweep_expired(&self) -> SweepResult {
        let mut guard = self.inner.write().await;
        let before = guard.len();
        guard.retain(|_, entry| !entry.is_expired(self.ttl));
        let after_ttl = guard.len();
        Self::enforce_capacity(&mut guard, self.max_entries);
        SweepResult {
            expired_removed: before.saturating_sub(after_ttl),
            overflow_removed: after_ttl.saturating_sub(guard.len()),
            remaining: guard.len(),
        }
    }

    fn enforce_capacity(map: &mut HashMap<String, CacheEntry>, max_entries: usize) {
        if map.len() <= max_entries {
            return;
        }

        let mut ordered: Vec<(String, Instant)> = map
            .iter()
            .map(|(key, entry)| (key.clone(), entry.created_at))
            .collect();
        ordered.sort_by_key(|(_, created_at)| *created_at);

        let overflow = map.len().saturating_sub(max_entries);
        for (key, _) in ordered.into_iter().take(overflow) {
            map.remove(&key);
        }
    }
}

#[derive(Clone, Debug)]
pub struct CachedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    response: CachedResponse,
    created_at: Instant,
}

impl CacheEntry {
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() >= ttl
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SweepResult {
    pub expired_removed: usize,
    pub overflow_removed: usize,
    pub remaining: usize,
}

#[derive(Clone, Debug)]
pub enum CachePurgeSelector {
    All,
    Prefix(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct CacheStats {
    pub entries: usize,
    pub expired_entries: usize,
    pub max_entries: usize,
    pub ttl_seconds: u64,
    pub oldest_age_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;
    use tokio::time::{sleep, Duration};

    fn sample_response() -> CachedResponse {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        CachedResponse {
            status: StatusCode::OK,
            headers,
            body: Bytes::from_static(br#"{"ok":true}"#),
        }
    }

    #[tokio::test]
    async fn expires_entries_after_ttl() {
        let cache = CacheStore::new(Duration::from_millis(20), 10);
        cache.insert("k".to_string(), sample_response()).await;
        sleep(Duration::from_millis(30)).await;

        assert!(cache.get("k").await.is_none());
    }

    #[tokio::test]
    async fn evicts_oldest_when_capacity_exceeded() {
        let cache = CacheStore::new(Duration::from_secs(60), 1);
        cache.insert("a".to_string(), sample_response()).await;
        sleep(Duration::from_millis(2)).await;
        cache.insert("b".to_string(), sample_response()).await;

        assert!(cache.get("a").await.is_none());
        assert!(cache.get("b").await.is_some());
    }

    #[tokio::test]
    async fn purges_by_prefix() {
        let cache = CacheStore::new(Duration::from_secs(60), 10);
        cache
            .insert(
                "POST:/worker/analyze/chunks:1".to_string(),
                sample_response(),
            )
            .await;
        cache
            .insert(
                "POST:/worker/analyze/skeleton:2".to_string(),
                sample_response(),
            )
            .await;

        let removed = cache
            .purge(CachePurgeSelector::Prefix(
                "POST:/worker/analyze/chunks".to_string(),
            ))
            .await;

        assert_eq!(removed, 1);
        assert!(cache.get("POST:/worker/analyze/chunks:1").await.is_none());
        assert!(cache.get("POST:/worker/analyze/skeleton:2").await.is_some());
    }
}
