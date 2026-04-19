use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{any, get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    admin, http_response::with_standard_headers, proxy, request_id::resolve_request_id,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/admin/cache", get(admin::cache_stats))
        .route("/admin/cache/purge", post(admin::purge_cache))
        .route("/api/*path", any(proxy::proxy_to_api))
        .route("/worker/*path", any(proxy::proxy_to_worker))
        .with_state(state)
}

async fn health(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    with_standard_headers(
        (
            StatusCode::OK,
            Json(HealthResponse {
                ok: true,
                api_base_url: state.config.api_base_url,
                worker_base_url: state.config.worker_base_url,
            }),
        )
            .into_response(),
        &request_id,
        "miss",
    )
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    api_base_url: String,
    worker_base_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::SocketAddr, time::Duration};

    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    use crate::{cache::CacheStore, config::Config, state::AppState};

    fn state() -> AppState {
        AppState {
            config: Config {
                http_addr: "127.0.0.1:8070".parse::<SocketAddr>().unwrap(),
                api_base_url: "http://127.0.0.1:8080".to_string(),
                worker_base_url: "http://127.0.0.1:8090".to_string(),
                admin_token: Some("secret".to_string()),
                upstream_timeout: Duration::from_secs(5),
                cache_ttl: Duration::from_secs(60),
                gc_interval: Duration::from_secs(60),
                cache_max_entries: 32,
                max_request_body_bytes: 1024,
            },
            http_client: reqwest::Client::new(),
            cache: CacheStore::new(Duration::from_secs(60), 32),
        }
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = build_router(state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"ok\":true"));
    }

    #[tokio::test]
    async fn cache_admin_requires_token() {
        let app = build_router(state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/cache")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn health_endpoint_returns_request_id_header() {
        let app = build_router(state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.headers().get("x-request-id").is_some());
        assert_eq!(response.headers().get("x-proxy-cache").unwrap(), "miss");
    }

    #[tokio::test]
    async fn cache_admin_accepts_valid_token() {
        let app = build_router(state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/cache")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("x-request-id").is_some());
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"entries\":0"));
    }

    #[tokio::test]
    async fn cache_purge_removes_target_scope() {
        let state = state();
        state
            .cache
            .insert(
                "POST:/worker/analyze/chunks:1".to_string(),
                crate::cache::CachedResponse {
                    status: StatusCode::OK,
                    headers: http::HeaderMap::new(),
                    body: bytes::Bytes::from_static(br#"{"ok":true}"#),
                },
            )
            .await;
        state
            .cache
            .insert(
                "POST:/worker/analyze/skeleton:2".to_string(),
                crate::cache::CachedResponse {
                    status: StatusCode::OK,
                    headers: http::HeaderMap::new(),
                    body: bytes::Bytes::from_static(br#"{"ok":true}"#),
                },
            )
            .await;

        let app = build_router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/cache/purge")
                    .header("content-type", "application/json")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::from(r#"{"scope":"chunks"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(state
            .cache
            .get("POST:/worker/analyze/chunks:1")
            .await
            .is_none());
        assert!(state
            .cache
            .get("POST:/worker/analyze/skeleton:2")
            .await
            .is_some());
    }
}
