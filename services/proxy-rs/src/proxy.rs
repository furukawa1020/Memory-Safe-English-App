use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri},
};
use bytes::Bytes;
use reqwest::Method;
use sha2::{Digest, Sha256};

use crate::{
    cache::CachedResponse,
    client_ip::resolve_client_ip,
    request_guard::{validate_request, GuardUpstream},
    rate_limit,
    request_id::resolve_request_id,
    response_headers::{apply_standard_headers, apply_upstream_header},
    security_audit::log_event,
    state::AppState,
};

static HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub async fn proxy_to_api(State(state): State<AppState>, request: Request<Body>) -> Response<Body> {
    forward(state, request, Upstream::Api).await
}

pub async fn proxy_to_worker(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Response<Body> {
    forward(state, request, Upstream::Worker).await
}

async fn forward(state: AppState, request: Request<Body>, upstream: Upstream) -> Response<Body> {
    let client_ip = resolve_client_ip(&request, &state.config.trusted_proxy_ips);
    let method = request.method().clone();
    let request_id = resolve_request_id(request.headers());
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/")
        .to_string();

    if let Some(rejection) = validate_request(
        upstream.guard_upstream(),
        &method,
        &path_and_query,
        request.headers(),
    ) {
        log_event(
            rejection.event,
            request_id.to_str().unwrap_or("proxy-request-id"),
            &client_ip,
            &path_and_query,
            rejection.message,
        );
        return error_response(
            rejection.status,
            rejection.message,
            &request_id,
            upstream.header_value(),
        );
    }

    if upstream == Upstream::Api && rate_limit::is_auth_path(&method, &path_and_query) {
        let decision = state
            .auth_rate_limiter
            .allow(&auth_rate_limit_key(&path_and_query, &client_ip))
            .await;
        if !decision.allowed {
            log_event(
                "proxy_auth_rate_limited",
                request_id.to_str().unwrap_or("proxy-request-id"),
                &client_ip,
                &path_and_query,
                "too many authentication attempts",
            );
            return rate_limited_response(
                &request_id,
                Upstream::ProxyRateLimited.header_value(),
                decision.retry_after,
            );
        }
    }

    let (parts, body) = request.into_parts();

    let body_bytes = match to_bytes(body, state.config.max_request_body_bytes).await {
        Ok(bytes) => bytes,
        Err(_) => {
            log_event(
                "proxy_body_too_large",
                request_id.to_str().unwrap_or("proxy-request-id"),
                &client_ip,
                &path_and_query,
                "request body exceeded proxy limit",
            );
            return error_response(
                StatusCode::PAYLOAD_TOO_LARGE,
                "request body too large",
                &request_id,
                upstream.header_value(),
            )
        }
    };

    let maybe_cache_key = cache_key(&upstream, &method, &path_and_query, &body_bytes);
    if let Some(key) = maybe_cache_key.as_ref() {
        if let Some(cached) = state.cache.get(key).await {
            return build_cached_response(cached, &request_id, upstream.cache_header_value());
        }
    }

    let upstream_url = match upstream_url(&state, &upstream, &path_and_query) {
        Ok(url) => url,
        Err(_) => {
            log_event(
                "proxy_upstream_path_rejected",
                request_id.to_str().unwrap_or("proxy-request-id"),
                &client_ip,
                &path_and_query,
                "request path could not be rewritten for upstream",
            );
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid upstream request path",
                &request_id,
                upstream.header_value(),
            )
        }
    };

    let request_headers = sanitize_request_headers(&parts.headers, &request_id, &client_ip);
    let reqwest_method = Method::from_bytes(method.as_str().as_bytes()).unwrap_or(Method::GET);
    let upstream_response = match state
        .http_client
        .request(reqwest_method, upstream_url)
        .headers(request_headers)
        .body(body_bytes.clone())
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            log_event(
                "proxy_upstream_failed",
                request_id.to_str().unwrap_or("proxy-request-id"),
                &client_ip,
                &path_and_query,
                "upstream request failed",
            );
            return error_response(
                StatusCode::BAD_GATEWAY,
                "upstream request failed",
                &request_id,
                upstream.header_value(),
            )
        }
    };

    let status = StatusCode::from_u16(upstream_response.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let headers = sanitize_response_headers(
        upstream_response.headers(),
        &request_id,
        upstream.header_value(),
    );
    let response_body = match upstream_response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            log_event(
                "proxy_upstream_read_failed",
                request_id.to_str().unwrap_or("proxy-request-id"),
                &client_ip,
                &path_and_query,
                "failed to read upstream response body",
            );
            return error_response(
                StatusCode::BAD_GATEWAY,
                "failed to read upstream response",
                &request_id,
                upstream.header_value(),
            )
        }
    };

    if let Some(key) = maybe_cache_key {
        if status.is_success() {
            state
                .cache
                .insert(
                    key,
                    CachedResponse {
                        status,
                        headers: headers.clone(),
                        body: response_body.clone(),
                    },
                )
                .await;
        }
    }

    build_response(status, headers, response_body)
}

fn upstream_url(
    state: &AppState,
    upstream: &Upstream,
    path_and_query: &str,
) -> Result<String, http::uri::InvalidUri> {
    let uri: Uri = path_and_query.parse()?;
    let suffix = uri
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");
    let without_prefix = suffix.trim_start_matches(upstream.route_prefix());
    let rewritten = if without_prefix.starts_with('/') {
        without_prefix.to_string()
    } else {
        format!("/{without_prefix}")
    };

    let base = match upstream {
        Upstream::Api => &state.config.api_base_url,
        Upstream::ProxyRateLimited => {
            unreachable!("proxy rate-limited responses are handled without upstream forwarding")
        }
        Upstream::Worker => &state.config.worker_base_url,
    };

    Ok(format!("{base}{rewritten}"))
}

fn cache_key(
    upstream: &Upstream,
    method: &http::Method,
    path_and_query: &str,
    body: &Bytes,
) -> Option<String> {
    if *upstream != Upstream::Worker || method != http::Method::POST {
        return None;
    }

    let normalized = path_and_query.split('?').next().unwrap_or(path_and_query);
    if normalized != "/worker/analyze/chunks" && normalized != "/worker/analyze/skeleton" {
        return None;
    }

    let mut hasher = Sha256::new();
    hasher.update(body);
    let digest = format!("{:x}", hasher.finalize());
    Some(format!("{}:{normalized}:{digest}", method.as_str()))
}

fn sanitize_request_headers(
    headers: &HeaderMap,
    request_id: &HeaderValue,
    client_ip: &str,
) -> reqwest::header::HeaderMap {
    let mut sanitized = reqwest::header::HeaderMap::new();
    for (name, value) in headers {
        if should_skip_header(name.as_str())
            || name.as_str().eq_ignore_ascii_case("host")
            || name.as_str().eq_ignore_ascii_case("x-forwarded-for")
            || name.as_str().eq_ignore_ascii_case("x-real-ip")
        {
            continue;
        }
        sanitized.insert(name.clone(), value.clone());
    }
    sanitized.insert(HeaderName::from_static("x-request-id"), request_id.clone());
    if !client_ip.is_empty()
        && client_ip != "unknown"
        && !sanitized.contains_key(HeaderName::from_static("x-forwarded-for"))
    {
        if let Ok(value) = HeaderValue::from_str(client_ip) {
            sanitized.insert(HeaderName::from_static("x-forwarded-for"), value);
        }
    }
    if !client_ip.is_empty() && client_ip != "unknown" {
        if let Ok(value) = HeaderValue::from_str(client_ip) {
            sanitized.insert(HeaderName::from_static("x-real-ip"), value);
        }
    }
    sanitized
}

fn sanitize_response_headers(
    headers: &reqwest::header::HeaderMap,
    request_id: &HeaderValue,
    upstream_name: &'static str,
) -> HeaderMap {
    let mut sanitized = HeaderMap::new();
    for (name, value) in headers {
        if should_skip_header(name.as_str()) {
            continue;
        }
        sanitized.insert(name.clone(), value.clone());
    }
    apply_standard_headers(&mut sanitized, request_id, "miss");
    apply_upstream_header(&mut sanitized, upstream_name);
    sanitized
}

fn build_cached_response(
    mut response: CachedResponse,
    request_id: &HeaderValue,
    upstream_name: &'static str,
) -> Response<Body> {
    apply_standard_headers(&mut response.headers, request_id, "hit");
    apply_upstream_header(&mut response.headers, upstream_name);
    build_response(response.status, response.headers, response.body)
}

fn build_response(status: StatusCode, headers: HeaderMap, body: Bytes) -> Response<Body> {
    let mut response = Response::new(Body::from(body));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}

fn error_response(
    status: StatusCode,
    message: &'static str,
    request_id: &HeaderValue,
    upstream_name: &'static str,
) -> Response<Body> {
    let payload = serde_json::json!({ "error": message }).to_string();
    let mut response = Response::new(Body::from(payload));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    apply_standard_headers(headers, request_id, "miss");
    apply_upstream_header(headers, upstream_name);
    response
}

fn rate_limited_response(
    request_id: &HeaderValue,
    upstream_name: &'static str,
    retry_after: std::time::Duration,
) -> Response<Body> {
    let mut response = error_response(
        StatusCode::TOO_MANY_REQUESTS,
        "too many authentication attempts",
        request_id,
        upstream_name,
    );
    let retry_after_seconds = retry_after.as_secs().max(1).to_string();
    if let Ok(value) = HeaderValue::from_str(&retry_after_seconds) {
        response
            .headers_mut()
            .insert(HeaderName::from_static("retry-after"), value);
    }
    response
}

fn auth_rate_limit_key(path_and_query: &str, client_ip: &str) -> String {
    let normalized = path_and_query.split('?').next().unwrap_or(path_and_query);
    format!("auth:{normalized}:{client_ip}")
}

fn should_skip_header(header_name: &str) -> bool {
    HOP_BY_HOP_HEADERS
        .iter()
        .any(|value| value.eq_ignore_ascii_case(header_name))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Upstream {
    Api,
    ProxyRateLimited,
    Worker,
}

impl Upstream {
    fn guard_upstream(&self) -> GuardUpstream {
        match self {
            Self::Api | Self::ProxyRateLimited => GuardUpstream::Api,
            Self::Worker => GuardUpstream::Worker,
        }
    }

    fn route_prefix(&self) -> &'static str {
        match self {
            Self::Api => "/api",
            Self::ProxyRateLimited => "",
            Self::Worker => "/worker",
        }
    }

    fn header_value(&self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::ProxyRateLimited => "proxy-auth-rate-limit",
            Self::Worker => "worker",
        }
    }

    fn cache_header_value(&self) -> &'static str {
        match self {
            Self::Api => "api-cache",
            Self::ProxyRateLimited => "proxy-auth-rate-limit",
            Self::Worker => "worker-cache",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{header::CONTENT_TYPE, HeaderValue};

    #[test]
    fn only_worker_analysis_posts_are_cacheable() {
        let body = Bytes::from_static(br#"{"text":"hello"}"#);
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::POST,
            "/worker/analyze/chunks",
            &body
        )
        .is_some());
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::POST,
            "/worker/analyze/skeleton",
            &body
        )
        .is_some());
        assert!(cache_key(
            &Upstream::Worker,
            &http::Method::GET,
            "/worker/analyze/chunks",
            &body
        )
        .is_none());
        assert!(cache_key(&Upstream::Api, &http::Method::POST, "/api/contents", &body).is_none());
    }

    #[test]
    fn sanitize_request_headers_sets_forwarded_for_when_missing() {
        let headers = HeaderMap::new();
        let request_id = HeaderValue::from_static("request-123");

        let sanitized = sanitize_request_headers(&headers, &request_id, "198.51.100.10");

        assert_eq!(sanitized.get("x-request-id").unwrap(), "request-123");
        assert_eq!(sanitized.get("x-forwarded-for").unwrap(), "198.51.100.10");
        assert_eq!(sanitized.get("x-real-ip").unwrap(), "198.51.100.10");
    }

    #[test]
    fn sanitize_request_headers_replaces_untrusted_forwarding_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.10"));
        headers.insert("x-real-ip", HeaderValue::from_static("203.0.113.11"));
        let request_id = HeaderValue::from_static("request-123");

        let sanitized = sanitize_request_headers(&headers, &request_id, "198.51.100.10");

        assert_eq!(sanitized.get("x-forwarded-for").unwrap(), "198.51.100.10");
        assert_eq!(sanitized.get("x-real-ip").unwrap(), "198.51.100.10");
    }

    #[test]
    fn sanitize_request_headers_keeps_json_content_type() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let request_id = HeaderValue::from_static("request-123");

        let sanitized = sanitize_request_headers(&headers, &request_id, "198.51.100.10");

        assert_eq!(sanitized.get(CONTENT_TYPE).unwrap(), "application/json");
    }
}
