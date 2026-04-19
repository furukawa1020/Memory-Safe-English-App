use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    cache::{CachePurgeSelector, CacheStats},
    request_id::resolve_request_id,
    response_headers::apply_standard_headers,
    state::AppState,
};

const ADMIN_TOKEN_HEADER: &str = "x-proxy-admin-token";

pub async fn cache_stats(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    if !is_authorized(&state, &headers) {
        return with_standard_headers(
            (
                StatusCode::UNAUTHORIZED,
                Json(AdminErrorResponse {
                    error: "unauthorized",
                }),
            )
                .into_response(),
            &request_id,
            "miss",
        );
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
    headers: HeaderMap,
    Json(request): Json<PurgeCacheRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    if !is_authorized(&state, &headers) {
        return with_standard_headers(
            (
                StatusCode::UNAUTHORIZED,
                Json(AdminErrorResponse {
                    error: "unauthorized",
                }),
            )
                .into_response(),
            &request_id,
            "miss",
        );
    }

    let selector = match request.scope.as_deref() {
        Some("all") | None => CachePurgeSelector::All,
        Some("chunks") => CachePurgeSelector::Prefix("POST:/worker/analyze/chunks".to_string()),
        Some("skeleton") => CachePurgeSelector::Prefix("POST:/worker/analyze/skeleton".to_string()),
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

fn is_authorized(state: &AppState, headers: &HeaderMap) -> bool {
    match state.config.admin_token.as_ref() {
        Some(expected) => headers
            .get(ADMIN_TOKEN_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(|value| value == expected)
            .unwrap_or(false),
        None => false,
    }
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

fn with_standard_headers(
    mut response: Response,
    request_id: &HeaderValue,
    cache_state: &'static str,
) -> Response {
    apply_standard_headers(response.headers_mut(), request_id, cache_state);
    response
}
