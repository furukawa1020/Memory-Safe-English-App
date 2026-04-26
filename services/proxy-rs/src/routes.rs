use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{any, get, post},
    Json, Router,
};
use serde::Serialize;

use crate::{
    admin, frontend, http_response::with_standard_headers, problems, proxy, readiness,
    request_id::resolve_request_id, state::AppState,
    response_headers::HeaderPolicy,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(readiness::ready))
        .route("/bootstrap/mobile", get(frontend::mobile_bootstrap))
        .route("/problem-bank", get(problems::list_problems))
        .route("/problem-bank/:id", get(problems::get_problem))
        .route("/problem-bank/generate", post(problems::generate_problems))
        .route("/admin/cache", get(admin::cache_stats))
        .route("/admin/cache/purge", post(admin::purge_cache))
        .route("/auth/*path", any(proxy::proxy_to_api))
        .route("/me", any(proxy::proxy_to_api))
        .route("/analysis/*path", any(proxy::proxy_to_api))
        .route("/contents", any(proxy::proxy_to_api))
        .route("/contents/*path", any(proxy::proxy_to_api))
        .route("/sessions/*path", any(proxy::proxy_to_api))
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
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
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
        routing::get,
        Json as AxumJson, Router as AxumRouter,
    };
    use tokio::net::TcpListener;
    use tower::ServiceExt;

    use crate::{
        cache::CacheStore,
        config::{Config, RuntimeEnvironment},
        rate_limit::RateLimiter,
        state::AppState,
    };

    fn state() -> AppState {
        AppState {
            config: Config {
                runtime_environment: RuntimeEnvironment::Development,
                http_addr: "127.0.0.1:8070".parse::<SocketAddr>().unwrap(),
                api_base_url: "http://127.0.0.1:8080".to_string(),
                worker_base_url: "http://127.0.0.1:8090".to_string(),
                admin_token: Some("secret".to_string()),
                trusted_proxy_ips: vec!["127.0.0.1".parse().unwrap()],
                admin_allowed_ips: Vec::new(),
                admin_rate_limit_max_requests: 30,
                admin_rate_limit_window: Duration::from_secs(60),
                auth_rate_limit_max_requests: 10,
                auth_rate_limit_window: Duration::from_secs(60),
                upstream_timeout: Duration::from_secs(5),
                cache_ttl: Duration::from_secs(60),
                gc_interval: Duration::from_secs(60),
                cache_max_entries: 32,
                max_request_body_bytes: 1024,
            },
            http_client: reqwest::Client::new(),
            cache: CacheStore::new(Duration::from_secs(60), 32),
            problem_bank: crate::problem_bank::ProblemBank::seeded(),
            admin_rate_limiter: RateLimiter::new(30, Duration::from_secs(60)),
            auth_rate_limiter: RateLimiter::new(10, Duration::from_secs(60)),
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
    async fn cache_admin_rejects_disallowed_ip() {
        let mut state = state();
        state.config.admin_allowed_ips = vec!["127.0.0.1".parse().unwrap()];
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/admin/cache")
                    .header("x-proxy-admin-token", "secret")
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn cache_admin_is_rate_limited() {
        let mut state = state();
        state.config.admin_rate_limit_max_requests = 1;
        state.admin_rate_limiter = RateLimiter::new(1, Duration::from_secs(60));
        let app = build_router(state);

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/admin/cache")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(
                Request::builder()
                    .uri("/admin/cache")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(second.headers().get("retry-after").is_some());
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

    #[tokio::test]
    async fn cache_purge_rejects_unknown_json_fields() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/cache/purge")
                    .header("content-type", "application/json")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::from(r#"{"scope":"chunks","extra":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("invalid_json"));
    }

    #[tokio::test]
    async fn cache_purge_rejects_invalid_json_body() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/admin/cache/purge")
                    .header("content-type", "application/json")
                    .header("x-proxy-admin-token", "secret")
                    .body(Body::from(r#"{"scope":"chunks""#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("invalid_json"));
    }

    #[tokio::test]
    async fn ready_endpoint_returns_ok_when_upstreams_are_healthy() {
        let api = spawn_health_server(StatusCode::OK).await;
        let worker = spawn_health_server(StatusCode::OK).await;
        let app = build_router(state_with_urls(api.base_url(), worker.base_url()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 2048).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"ready\":true"));
    }

    #[tokio::test]
    async fn ready_endpoint_returns_service_unavailable_when_upstream_fails() {
        let api = spawn_health_server(StatusCode::OK).await;
        let worker = spawn_health_server(StatusCode::INTERNAL_SERVER_ERROR).await;
        let app = build_router(state_with_urls(api.base_url(), worker.base_url()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = to_bytes(response.into_body(), 2048).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"ready\":false"));
    }

    #[tokio::test]
    async fn mobile_friendly_auth_route_proxies_to_api() {
        let api = spawn_api_server().await;
        let worker = spawn_health_server(StatusCode::OK).await;
        let app = build_router(state_with_urls(api.base_url(), worker.base_url()));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"email":"reader@example.com","password":"secret123456"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("x-proxy-upstream").unwrap(), "api");
        let body = to_bytes(response.into_body(), 2048).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"access_token\":\"token-123\""));
    }

    #[tokio::test]
    async fn auth_route_is_rate_limited_before_reaching_api() {
        let api = spawn_api_server().await;
        let worker = spawn_health_server(StatusCode::OK).await;
        let app = build_router(state_with_urls_and_auth_limit(
            api.base_url(),
            worker.base_url(),
            1,
        ));

        let first = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", "198.51.100.10")
                    .body(Body::from(
                        r#"{"email":"reader@example.com","password":"secret123456"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/login")
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", "198.51.100.10")
                    .body(Body::from(
                        r#"{"email":"reader@example.com","password":"secret123456"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            second.headers().get("x-proxy-upstream").unwrap(),
            "proxy-auth-rate-limit"
        );
        assert!(second.headers().get("retry-after").is_some());
    }

    #[tokio::test]
    async fn mobile_bootstrap_returns_frontend_ready_metadata() {
        let api = spawn_health_server(StatusCode::OK).await;
        let worker = spawn_health_server(StatusCode::OK).await;
        let app = build_router(state_with_urls(api.base_url(), worker.base_url()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/bootstrap/mobile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"android_emulator\":\"http://10.0.2.2:8070\""));
        assert!(text.contains("\"login\":\"/auth/login\""));
        assert!(text.contains("\"refresh\":\"/auth/refresh\""));
        assert!(text.contains("\"reader_plan\":\"/analysis/reader-plan\""));
        assert!(text.contains("\"analytics_summary\":true"));
        assert!(text.contains("\"environment\":\"development\""));
    }

    #[tokio::test]
    async fn mobile_bootstrap_hides_internal_details_in_production() {
        let api = spawn_health_server(StatusCode::OK).await;
        let worker = spawn_health_server(StatusCode::OK).await;
        let mut state = state_with_urls(api.base_url(), worker.base_url());
        state.config.runtime_environment = RuntimeEnvironment::Production;
        state.config.admin_token = Some("0123456789abcdef".to_string());
        state.config.trusted_proxy_ips = vec!["127.0.0.1".parse().unwrap()];
        state.config.admin_allowed_ips = vec!["127.0.0.1".parse().unwrap()];
        let app = build_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/bootstrap/mobile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"environment\":\"production\""));
        assert!(text.contains("\"recommended_base_urls\":null"));
        assert!(text.contains("\"api\":null"));
        assert!(text.contains("\"worker\":null"));
        assert!(text.contains("\"login\":\"/auth/login\""));
    }

    #[tokio::test]
    async fn problem_bank_lists_seeded_items() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank?mode=speaking&limit=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"total\":2"));
        assert!(text.contains("\"mode\":\"speaking\""));
    }

    #[tokio::test]
    async fn problem_bank_returns_item_by_id() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/pb_read_001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"id\":\"pb_read_001\""));
        assert!(text.contains("\"wm_support\""));
    }

    #[tokio::test]
    async fn problem_bank_generates_problems_from_text() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/generate")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"text":"The client approved the design draft, but the delivery schedule is still under review.","target_context":"meeting","level_band":"toeic_750_800"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"target_context\":\"meeting\""));
        assert!(text.contains("\"mode\":\"reading\""));
        assert!(text.contains("\"mode\":\"listening\""));
        assert!(text.contains("\"mode\":\"speaking\""));
        assert!(text.contains("\"mode\":\"rescue\""));
    }

    fn state_with_urls(api_base_url: String, worker_base_url: String) -> AppState {
        state_with_urls_and_auth_limit(api_base_url, worker_base_url, 10)
    }

    fn state_with_urls_and_auth_limit(
        api_base_url: String,
        worker_base_url: String,
        auth_rate_limit_max_requests: usize,
    ) -> AppState {
        AppState {
            config: Config {
                runtime_environment: RuntimeEnvironment::Development,
                http_addr: "127.0.0.1:8070".parse::<SocketAddr>().unwrap(),
                api_base_url,
                worker_base_url,
                admin_token: Some("secret".to_string()),
                trusted_proxy_ips: vec!["127.0.0.1".parse().unwrap()],
                admin_allowed_ips: Vec::new(),
                admin_rate_limit_max_requests: 30,
                admin_rate_limit_window: Duration::from_secs(60),
                auth_rate_limit_max_requests,
                auth_rate_limit_window: Duration::from_secs(60),
                upstream_timeout: Duration::from_secs(5),
                cache_ttl: Duration::from_secs(60),
                gc_interval: Duration::from_secs(60),
                cache_max_entries: 32,
                max_request_body_bytes: 1024,
            },
            http_client: reqwest::Client::new(),
            cache: CacheStore::new(Duration::from_secs(60), 32),
            problem_bank: crate::problem_bank::ProblemBank::seeded(),
            admin_rate_limiter: RateLimiter::new(30, Duration::from_secs(60)),
            auth_rate_limiter: RateLimiter::new(
                auth_rate_limit_max_requests,
                Duration::from_secs(60),
            ),
        }
    }

    async fn spawn_health_server(status: StatusCode) -> TestServer {
        let app = AxumRouter::new().route(
            "/health",
            get(move || async move {
                (
                    status,
                    AxumJson(serde_json::json!({ "ok": status.is_success() })),
                )
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        TestServer { address }
    }

    async fn spawn_api_server() -> TestServer {
        let app = AxumRouter::new().route(
            "/auth/login",
            axum::routing::post(|| async {
                (
                    StatusCode::OK,
                    AxumJson(serde_json::json!({
                        "tokens": {
                            "access_token": "token-123",
                            "refresh_token": "refresh-123",
                            "token_type": "Bearer"
                        }
                    })),
                )
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        TestServer { address }
    }

    struct TestServer {
        address: SocketAddr,
    }

    impl TestServer {
        fn base_url(&self) -> String {
            format!("http://{}", self.address)
        }
    }
}
