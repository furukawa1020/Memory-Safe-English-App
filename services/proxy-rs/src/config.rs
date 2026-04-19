use std::{env, net::SocketAddr, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub http_addr: SocketAddr,
    pub api_base_url: String,
    pub worker_base_url: String,
    pub upstream_timeout: Duration,
    pub cache_ttl: Duration,
    pub gc_interval: Duration,
    pub cache_max_entries: usize,
    pub max_request_body_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            http_addr: parse_env("PROXY_HTTP_ADDR", "127.0.0.1:8070")?,
            api_base_url: parse_url("PROXY_API_BASE_URL", "http://127.0.0.1:8080"),
            worker_base_url: parse_url("PROXY_WORKER_BASE_URL", "http://127.0.0.1:8090"),
            upstream_timeout: Duration::from_secs(parse_env("PROXY_UPSTREAM_TIMEOUT_SECONDS", "10")?),
            cache_ttl: Duration::from_secs(parse_env("PROXY_CACHE_TTL_SECONDS", "300")?),
            gc_interval: Duration::from_secs(parse_env("PROXY_GC_INTERVAL_SECONDS", "60")?),
            cache_max_entries: parse_env("PROXY_CACHE_MAX_ENTRIES", "1024")?,
            max_request_body_bytes: parse_env("PROXY_MAX_REQUEST_BODY_BYTES", "262144")?,
        })
    }
}

fn parse_url(key: &str, default_value: &str) -> String {
    env::var(key)
        .unwrap_or_else(|_| default_value.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn parse_env<T>(key: &str, default_value: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    let raw = env::var(key).unwrap_or_else(|_| default_value.to_string());
    raw.parse::<T>()
        .map_err(|_| ConfigError::InvalidValue(key.to_string(), raw))
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid configuration value for {0}: {1}")]
    InvalidValue(String, String),
}
