use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    buckets: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn allow(&self, key: &str) -> RateLimitDecision {
        self.allow_at(key, Instant::now()).await
    }

    async fn allow_at(&self, key: &str, now: Instant) -> RateLimitDecision {
        if key.is_empty() || self.max_requests == 0 {
            return RateLimitDecision::allowed();
        }

        let window_start = now.checked_sub(self.window).unwrap_or(now);
        let mut buckets = self.buckets.lock().await;
        let events = buckets.entry(key.to_string()).or_default();
        events.retain(|event| *event >= window_start);

        if events.len() >= self.max_requests {
            let retry_after = events
                .first()
                .map(|event| event.checked_add(self.window).unwrap_or(*event))
                .map(|deadline| deadline.saturating_duration_since(now))
                .unwrap_or(Duration::from_secs(1))
                .max(Duration::from_secs(1));

            return RateLimitDecision {
                allowed: false,
                retry_after,
            };
        }

        events.push(now);
        RateLimitDecision::allowed()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub retry_after: Duration,
}

impl RateLimitDecision {
    fn allowed() -> Self {
        Self {
            allowed: true,
            retry_after: Duration::ZERO,
        }
    }
}

pub fn is_auth_path(method: &http::Method, path: &str) -> bool {
    if method != http::Method::POST {
        return false;
    }

    let normalized = path.split('?').next().unwrap_or(path);
    matches!(
        normalized,
        "/auth/login"
            | "/auth/register"
            | "/auth/refresh"
            | "/api/auth/login"
            | "/api/auth/register"
            | "/api/auth/refresh"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn limiter_blocks_after_capacity() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        let now = Instant::now();

        let first = limiter.allow_at("client:198.51.100.10", now).await;
        assert!(first.allowed);

        let second = limiter
            .allow_at("client:198.51.100.10", now + Duration::from_secs(1))
            .await;
        assert!(!second.allowed);
        assert!(second.retry_after >= Duration::from_secs(1));
    }

    #[test]
    fn auth_path_matcher_only_targets_auth_posts() {
        assert!(is_auth_path(&http::Method::POST, "/auth/login"));
        assert!(is_auth_path(&http::Method::POST, "/api/auth/register"));
        assert!(!is_auth_path(&http::Method::GET, "/auth/login"));
        assert!(!is_auth_path(&http::Method::POST, "/contents"));
    }
}
