use reqwest::Client;

use crate::{cache::CacheStore, config::Config, problem_bank::ProblemBank, rate_limit::RateLimiter};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: Client,
    pub cache: CacheStore,
    pub problem_bank: ProblemBank,
    pub admin_rate_limiter: RateLimiter,
    pub auth_rate_limiter: RateLimiter,
}
