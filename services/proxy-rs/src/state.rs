use reqwest::Client;

use crate::{cache::CacheStore, config::Config, rate_limit::RateLimiter};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: Client,
    pub cache: CacheStore,
    pub admin_rate_limiter: RateLimiter,
    pub auth_rate_limiter: RateLimiter,
}
