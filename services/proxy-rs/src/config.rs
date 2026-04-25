use std::{
    env,
    net::{IpAddr, SocketAddr},
    time::Duration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeEnvironment {
    Development,
    Production,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub runtime_environment: RuntimeEnvironment,
    pub http_addr: SocketAddr,
    pub api_base_url: String,
    pub worker_base_url: String,
    pub admin_token: Option<String>,
    pub trusted_proxy_ips: Vec<IpAddr>,
    pub admin_allowed_ips: Vec<IpAddr>,
    pub admin_rate_limit_max_requests: usize,
    pub admin_rate_limit_window: Duration,
    pub auth_rate_limit_max_requests: usize,
    pub auth_rate_limit_window: Duration,
    pub upstream_timeout: Duration,
    pub cache_ttl: Duration,
    pub gc_interval: Duration,
    pub cache_max_entries: usize,
    pub max_request_body_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = Self {
            runtime_environment: parse_runtime_environment("PROXY_RUNTIME_ENV", "development")?,
            http_addr: parse_env("PROXY_HTTP_ADDR", "127.0.0.1:8070")?,
            api_base_url: parse_url("PROXY_API_BASE_URL", "http://127.0.0.1:8080"),
            worker_base_url: parse_url("PROXY_WORKER_BASE_URL", "http://127.0.0.1:8090"),
            admin_token: parse_optional("PROXY_ADMIN_TOKEN"),
            trusted_proxy_ips: parse_ip_list("PROXY_TRUSTED_PROXY_IPS")?,
            admin_allowed_ips: parse_ip_list("PROXY_ADMIN_ALLOWED_IPS")?,
            admin_rate_limit_max_requests: parse_env("PROXY_ADMIN_RATE_LIMIT_MAX_REQUESTS", "30")?,
            admin_rate_limit_window: Duration::from_secs(parse_env(
                "PROXY_ADMIN_RATE_LIMIT_WINDOW_SECONDS",
                "60",
            )?),
            auth_rate_limit_max_requests: parse_env("PROXY_AUTH_RATE_LIMIT_MAX_REQUESTS", "20")?,
            auth_rate_limit_window: Duration::from_secs(parse_env(
                "PROXY_AUTH_RATE_LIMIT_WINDOW_SECONDS",
                "60",
            )?),
            upstream_timeout: Duration::from_secs(parse_env(
                "PROXY_UPSTREAM_TIMEOUT_SECONDS",
                "10",
            )?),
            cache_ttl: Duration::from_secs(parse_env("PROXY_CACHE_TTL_SECONDS", "300")?),
            gc_interval: Duration::from_secs(parse_env("PROXY_GC_INTERVAL_SECONDS", "60")?),
            cache_max_entries: parse_env("PROXY_CACHE_MAX_ENTRIES", "1024")?,
            max_request_body_bytes: parse_env("PROXY_MAX_REQUEST_BODY_BYTES", "262144")?,
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.auth_rate_limit_max_requests == 0 {
            return Err(ConfigError::InvalidValue(
                "PROXY_AUTH_RATE_LIMIT_MAX_REQUESTS".to_string(),
                self.auth_rate_limit_max_requests.to_string(),
            ));
        }
        if self.admin_rate_limit_max_requests == 0 {
            return Err(ConfigError::InvalidValue(
                "PROXY_ADMIN_RATE_LIMIT_MAX_REQUESTS".to_string(),
                self.admin_rate_limit_max_requests.to_string(),
            ));
        }
        if self.admin_rate_limit_window <= Duration::ZERO {
            return Err(ConfigError::InvalidValue(
                "PROXY_ADMIN_RATE_LIMIT_WINDOW_SECONDS".to_string(),
                self.admin_rate_limit_window.as_secs().to_string(),
            ));
        }
        if self.auth_rate_limit_window <= Duration::ZERO {
            return Err(ConfigError::InvalidValue(
                "PROXY_AUTH_RATE_LIMIT_WINDOW_SECONDS".to_string(),
                self.auth_rate_limit_window.as_secs().to_string(),
            ));
        }
        if self.upstream_timeout <= Duration::ZERO {
            return Err(ConfigError::InvalidValue(
                "PROXY_UPSTREAM_TIMEOUT_SECONDS".to_string(),
                self.upstream_timeout.as_secs().to_string(),
            ));
        }
        if self.cache_ttl <= Duration::ZERO {
            return Err(ConfigError::InvalidValue(
                "PROXY_CACHE_TTL_SECONDS".to_string(),
                self.cache_ttl.as_secs().to_string(),
            ));
        }
        if self.gc_interval <= Duration::ZERO {
            return Err(ConfigError::InvalidValue(
                "PROXY_GC_INTERVAL_SECONDS".to_string(),
                self.gc_interval.as_secs().to_string(),
            ));
        }
        if self.cache_max_entries == 0 {
            return Err(ConfigError::InvalidValue(
                "PROXY_CACHE_MAX_ENTRIES".to_string(),
                self.cache_max_entries.to_string(),
            ));
        }
        if self.max_request_body_bytes == 0 {
            return Err(ConfigError::InvalidValue(
                "PROXY_MAX_REQUEST_BODY_BYTES".to_string(),
                self.max_request_body_bytes.to_string(),
            ));
        }
        if let Some(admin_token) = self.admin_token.as_ref() {
            if admin_token.len() < 16 {
                return Err(ConfigError::InvalidValue(
                    "PROXY_ADMIN_TOKEN".to_string(),
                    "too_short".to_string(),
                ));
            }
        }
        if self.runtime_environment == RuntimeEnvironment::Production {
            if self.admin_token.is_none() {
                return Err(ConfigError::MissingRequired(
                    "PROXY_ADMIN_TOKEN".to_string(),
                    "production requires an admin token".to_string(),
                ));
            }
            if self.admin_allowed_ips.is_empty() {
                return Err(ConfigError::MissingRequired(
                    "PROXY_ADMIN_ALLOWED_IPS".to_string(),
                    "production requires an explicit admin IP allowlist".to_string(),
                ));
            }
            if self.trusted_proxy_ips.is_empty() {
                return Err(ConfigError::MissingRequired(
                    "PROXY_TRUSTED_PROXY_IPS".to_string(),
                    "production requires explicit trusted proxy peers".to_string(),
                ));
            }
            if self.auth_rate_limit_max_requests > 100 {
                return Err(ConfigError::InvalidValue(
                    "PROXY_AUTH_RATE_LIMIT_MAX_REQUESTS".to_string(),
                    self.auth_rate_limit_max_requests.to_string(),
                ));
            }
            if self.admin_rate_limit_max_requests > 60 {
                return Err(ConfigError::InvalidValue(
                    "PROXY_ADMIN_RATE_LIMIT_MAX_REQUESTS".to_string(),
                    self.admin_rate_limit_max_requests.to_string(),
                ));
            }
            if self.max_request_body_bytes > 1024 * 1024 {
                return Err(ConfigError::InvalidValue(
                    "PROXY_MAX_REQUEST_BODY_BYTES".to_string(),
                    self.max_request_body_bytes.to_string(),
                ));
            }
            if self.api_base_url.is_empty() || self.worker_base_url.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "PROXY_API_BASE_URL/PROXY_WORKER_BASE_URL".to_string(),
                    "empty".to_string(),
                ));
            }
        }
        Ok(())
    }
}

fn parse_url(key: &str, default_value: &str) -> String {
    env::var(key)
        .unwrap_or_else(|_| default_value.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn parse_optional(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_ip_list(key: &str) -> Result<Vec<IpAddr>, ConfigError> {
    let raw = match env::var(key) {
        Ok(value) => value,
        Err(_) => return Ok(Vec::new()),
    };

    let mut values = Vec::new();
    for item in raw.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ip = trimmed
            .parse::<IpAddr>()
            .map_err(|_| ConfigError::InvalidValue(key.to_string(), trimmed.to_string()))?;
        values.push(ip);
    }
    Ok(values)
}

fn parse_env<T>(key: &str, default_value: &str) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    let raw = env::var(key).unwrap_or_else(|_| default_value.to_string());
    raw.parse::<T>()
        .map_err(|_| ConfigError::InvalidValue(key.to_string(), raw))
}

fn parse_runtime_environment(key: &str, default_value: &str) -> Result<RuntimeEnvironment, ConfigError> {
    let raw = env::var(key).unwrap_or_else(|_| default_value.to_string());
    match raw.trim().to_ascii_lowercase().as_str() {
        "development" | "dev" => Ok(RuntimeEnvironment::Development),
        "production" | "prod" => Ok(RuntimeEnvironment::Production),
        _ => Err(ConfigError::InvalidValue(key.to_string(), raw)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid configuration value for {0}: {1}")]
    InvalidValue(String, String),
    #[error("missing required configuration {0}: {1}")]
    MissingRequired(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_short_admin_token() {
        let config = Config {
            runtime_environment: RuntimeEnvironment::Development,
            http_addr: "127.0.0.1:8070".parse().unwrap(),
            api_base_url: "http://127.0.0.1:8080".to_string(),
            worker_base_url: "http://127.0.0.1:8090".to_string(),
            admin_token: Some("short".to_string()),
            trusted_proxy_ips: Vec::new(),
            admin_allowed_ips: Vec::new(),
            admin_rate_limit_max_requests: 30,
            admin_rate_limit_window: Duration::from_secs(60),
            auth_rate_limit_max_requests: 20,
            auth_rate_limit_window: Duration::from_secs(60),
            upstream_timeout: Duration::from_secs(10),
            cache_ttl: Duration::from_secs(300),
            gc_interval: Duration::from_secs(60),
            cache_max_entries: 1024,
            max_request_body_bytes: 262144,
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue(key, _)) if key == "PROXY_ADMIN_TOKEN"
        ));
    }

    #[test]
    fn production_requires_admin_allowlist_and_trusted_proxies() {
        let config = Config {
            runtime_environment: RuntimeEnvironment::Production,
            http_addr: "0.0.0.0:8070".parse().unwrap(),
            api_base_url: "http://api.internal:8080".to_string(),
            worker_base_url: "http://worker.internal:8090".to_string(),
            admin_token: Some("0123456789abcdef".to_string()),
            trusted_proxy_ips: Vec::new(),
            admin_allowed_ips: Vec::new(),
            admin_rate_limit_max_requests: 30,
            admin_rate_limit_window: Duration::from_secs(60),
            auth_rate_limit_max_requests: 20,
            auth_rate_limit_window: Duration::from_secs(60),
            upstream_timeout: Duration::from_secs(10),
            cache_ttl: Duration::from_secs(300),
            gc_interval: Duration::from_secs(60),
            cache_max_entries: 1024,
            max_request_body_bytes: 262144,
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::MissingRequired(key, _)) if key == "PROXY_ADMIN_ALLOWED_IPS"
        ));
    }

    #[test]
    fn production_rejects_excessive_body_limit() {
        let config = Config {
            runtime_environment: RuntimeEnvironment::Production,
            http_addr: "0.0.0.0:8070".parse().unwrap(),
            api_base_url: "http://api.internal:8080".to_string(),
            worker_base_url: "http://worker.internal:8090".to_string(),
            admin_token: Some("0123456789abcdef".to_string()),
            trusted_proxy_ips: vec!["10.0.0.1".parse().unwrap()],
            admin_allowed_ips: vec!["10.0.0.2".parse().unwrap()],
            admin_rate_limit_max_requests: 30,
            admin_rate_limit_window: Duration::from_secs(60),
            auth_rate_limit_max_requests: 20,
            auth_rate_limit_window: Duration::from_secs(60),
            upstream_timeout: Duration::from_secs(10),
            cache_ttl: Duration::from_secs(300),
            gc_interval: Duration::from_secs(60),
            cache_max_entries: 1024,
            max_request_body_bytes: 2 * 1024 * 1024,
        };

        assert!(matches!(
            config.validate(),
            Err(ConfigError::InvalidValue(key, _)) if key == "PROXY_MAX_REQUEST_BODY_BYTES"
        ));
    }
}
