use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, State},
    http::{HeaderMap, HeaderValue, Request, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::{
    cache::{CachePurgeSelector, CacheStats},
    client_ip::resolve_client_ip_from_parts,
    http_response::with_standard_headers,
    request_id::resolve_request_id,
    security_audit::log_event,
    state::AppState,
};

const ADMIN_TOKEN_HEADER: &str = "x-proxy-admin-token";

pub async fn cache_stats(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let client_ip = resolve_client_ip_from_parts(
        &headers,
        connect_info.as_ref().map(|info| info.0.ip()),
        &state.config.trusted_proxy_ips,
    );

    if let Some(response) = guard_admin_request(
        &state,
        &headers,
        &request_id,
        &client_ip,
        "/admin/cache",
    )
    .await
    {
        return response;
    }

    let stats = state.cache.stats().await;
    with_standard_headers(
        (StatusCode::OK, Json(CacheStatsResponse::from_stats(stats))).into_response(),
        &request_id,
        "miss",
    )
}

pub async fn purge_cache(
    State(state): State<AppState>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    headers: HeaderMap,
    Json(request): Json<PurgeCacheRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let client_ip = resolve_client_ip_from_parts(
        &headers,
        connect_info.as_ref().map(|info| info.0.ip()),
        &state.config.trusted_proxy_ips,
    );

    if let Some(response) = guard_admin_request(
        &state,
        &headers,
        &request_id,
        &client_ip,
        "/admin/cache/purge",
    )
    .await
    {
        return response;
    }

    let selector = match request.scope.as_deref() {
        Some("all") | None => CachePurgeSelector::All,
        Some("chunks") => CachePurgeSelector::Prefix("POST:/worker/analyze/chunks".to_string()),
        Some("skeleton") => {
            CachePurgeSelector::Prefix("POST:/worker/analyze/skeleton".to_string())
        }
        Some(_) => {
            return with_standard_headers(
                (
                    StatusCode::BAD_REQUEST,
                    Json(AdminErrorResponse {
                        error: "invalid purge scope",
                    }),
                )
                    .into_response(),
                &request_id,
                "miss",
            );
        }
    };

    let removed = state.cache.purge(selector).await;
    with_standard_headers(
        (StatusCode::OK, Json(PurgeCacheResponse { removed })).into_response(),
        &request_id,
        "miss",
    )
}

async fn guard_admin_request(
    state: &AppState,
    headers: &HeaderMap,
    request_id: &HeaderValue,
    client_ip: &str,
    path: &'static str,
) -> Option<http::Response<Body>> {
    if !is_allowed_ip(state, client_ip) {
        log_event(
            "admin_ip_rejected",
            request_id.to_str().unwrap_or("proxy-request-id"),
            client_ip,
            path,
            "admin client ip not allowed",
        );
        return Some(with_standard_headers(
            (
                StatusCode::FORBIDDEN,
                Json(AdminErrorResponse {
                    error: "forbidden",
                }),
            )
                .into_response(),
            request_id,
            "miss",
        ));
    }

    let rate_limit_key = format!("admin:{client_ip}:{path}");
    let decision = state.admin_rate_limiter.allow(&rate_limit_key).await;
    if !decision.allowed {
        log_event(
            "admin_rate_limited",
            request_id.to_str().unwrap_or("proxy-request-id"),
            client_ip,
            path,
            "admin rate limit exceeded",
        );
        return Some(rate_limited_response(request_id));
    }

    if !is_authorized(state, headers) {
        log_event(
            "admin_unauthorized",
            request_id.to_str().unwrap_or("proxy-request-id"),
            client_ip,
            path,
            "invalid admin token",
        );
        return Some(with_standard_headers(
            (
                StatusCode::UNAUTHORIZED,
                Json(AdminErrorResponse {
                    error: "unauthorized",
                }),
            )
                .into_response(),
            request_id,
            "miss",
        ));
    }

    None
}

fn is_allowed_ip(state: &AppState, client_ip: &str) -> bool {
    if state.config.admin_allowed_ips.is_empty() {
        return true;
    }

    let parsed = match client_ip.parse::<std::net::IpAddr>() {
        Ok(value) => value,
        Err(_) => return false,
    };
    state.config.admin_allowed_ips.contains(&parsed)
}

fn is_authorized(state: &AppState, headers: &HeaderMap) -> bool {
    match state.config.admin_token.as_ref() {
        Some(expected) => headers
            .get(ADMIN_TOKEN_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.as_bytes().ct_eq(expected.as_bytes()).into())
            .unwrap_or(false),
        None => false,
    }
}

fn rate_limited_response(request_id: &HeaderValue) -> http::Response<Body> {
    let mut response = with_standard_headers(
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(AdminErrorResponse {
                error: "rate_limited",
            }),
        )
            .into_response(),
        request_id,
        "miss",
    );
    response.headers_mut().insert(
        http::header::RETRY_AFTER,
        HeaderValue::from_static("1"),
    );
    response
}

#[derive(Deserialize)]
pub struct PurgeCacheRequest {
    pub scope: Option<String>,
}

#[derive(Serialize)]
struct CacheStatsResponse {
    entries: usize,
    expired_entries: usize,
    max_entries: usize,
    ttl_seconds: u64,
    oldest_age_seconds: u64,
}

impl CacheStatsResponse {
    fn from_stats(stats: CacheStats) -> Self {
        Self {
            entries: stats.entries,
            expired_entries: stats.expired_entries,
            max_entries: stats.max_entries,
            ttl_seconds: stats.ttl_seconds,
            oldest_age_seconds: stats.oldest_age_seconds,
        }
    }
}

#[derive(Serialize)]
struct PurgeCacheResponse {
    removed: usize,
}

#[derive(Serialize)]
struct AdminErrorResponse {
    error: &'static str,
}
