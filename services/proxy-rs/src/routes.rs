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
        .route("/problem-bank/custom", get(problems::list_custom_problems))
        .route("/problem-bank/activity", get(problems::problem_activity))
        .route("/problem-bank/dashboard", get(problems::problem_dashboard))
        .route("/problem-bank/insights", get(problems::problem_insights))
        .route("/problem-bank/snapshots", get(problems::list_snapshots))
        .route("/problem-bank/snapshots/compare", get(problems::compare_snapshots))
        .route("/problem-bank/snapshots/capture", post(problems::capture_snapshot))
        .route("/problem-bank/snapshots/:id", axum::routing::delete(problems::delete_snapshot))
        .route("/problem-bank/stale", get(problems::stale_problems))
        .route("/problem-bank/recommend", get(problems::recommend_problems))
        .route("/problem-bank/review-queue", get(problems::review_queue))
        .route("/problem-bank/weakness-queue", get(problems::weakness_queue))
        .route("/problem-bank/stats", get(problems::problem_bank_stats))
        .route(
            "/problem-bank/:id",
            get(problems::get_problem)
                .patch(problems::update_problem)
                .delete(problems::delete_problem),
        )
        .route("/problem-bank/:id/history", get(problems::problem_history))
        .route("/problem-bank/:id/save", post(problems::clone_problem))
        .route("/problem-bank/:id/usage", post(problems::record_problem_usage))
        .route("/problem-bank/generate", post(problems::generate_problems))
        .route("/problem-bank/save", post(problems::save_generated_problems))
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
                problem_bank_path: None,
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

    #[tokio::test]
    async fn problem_bank_generate_uses_worker_plan_details_when_available() {
        let api = spawn_health_server(StatusCode::OK).await;
        let worker = spawn_problem_worker_server().await;
        let app = build_router(state_with_urls(api.base_url(), worker.base_url()));

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
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("Focus here first"));
        assert!(text.contains("checkpoint meaning before the next chunk at 0.85x speed"));
        assert!(text.contains("Start with: 'The decision is approved.'"));
        assert!(text.contains("Practice saying: 'Could you give me the decision first?'"));
    }

    #[tokio::test]
    async fn problem_bank_stats_returns_seed_counts() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/stats")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"seeded\":"));
        assert!(text.contains("\"by_mode\""));
    }

    #[tokio::test]
    async fn problem_bank_recommend_returns_ranked_items() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/recommend?preferred_mode=speaking&target_context=meeting&level_band=toeic_750_800&focus_tag=status_update")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"id\":\"pb_speak_002\""));
    }

    #[tokio::test]
    async fn problem_bank_review_queue_prioritizes_recent_failures() {
        let app = build_router(state());

        let struggling_save = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let struggling_body = to_bytes(struggling_save.into_body(), 4096).await.unwrap();
        let struggling_text = String::from_utf8(struggling_body.to_vec()).unwrap();
        let struggling_start = struggling_text
            .find("\"id\":\"saved_")
            .expect("saved struggling problem id");
        let struggling_value = &struggling_text[struggling_start + 6..];
        let struggling_end = struggling_value.find('"').expect("saved id end quote");
        let struggling_id = &struggling_value[..struggling_end];

        let mastered_save = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_read_001/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let mastered_body = to_bytes(mastered_save.into_body(), 4096).await.unwrap();
        let mastered_text = String::from_utf8(mastered_body.to_vec()).unwrap();
        let mastered_start = mastered_text
            .find("\"id\":\"saved_")
            .expect("saved mastered problem id");
        let mastered_value = &mastered_text[mastered_start + 6..];
        let mastered_end = mastered_value.find('"').expect("saved id end quote");
        let mastered_id = &mastered_value[..mastered_end];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{struggling_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":666666666,"append_note":"lost the second step"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        for occurred_at in [111111111_u64, 222222222_u64, 333333333_u64] {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/problem-bank/{mastered_id}/usage"))
                        .header("content-type", "application/json")
                        .body(Body::from(format!(
                            "{{\"successful\":true,\"occurred_at_unix\":{occurred_at},\"append_note\":\"stable\"}}"
                        )))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/review-queue?source=reviewed&avoid_mastered=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains(struggling_id));
        assert!(!text.contains(mastered_id));
    }

    #[tokio::test]
    async fn problem_bank_weakness_queue_groups_by_mode() {
        let app = build_router(state());

        let speaking_save = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let speaking_body = to_bytes(speaking_save.into_body(), 4096).await.unwrap();
        let speaking_text = String::from_utf8(speaking_body.to_vec()).unwrap();
        let speaking_start = speaking_text
            .find("\"id\":\"saved_")
            .expect("saved speaking problem id");
        let speaking_value = &speaking_text[speaking_start + 6..];
        let speaking_end = speaking_value.find('"').expect("saved id end quote");
        let speaking_id = &speaking_value[..speaking_end];

        let reading_save = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_read_001/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let reading_body = to_bytes(reading_save.into_body(), 4096).await.unwrap();
        let reading_text = String::from_utf8(reading_body.to_vec()).unwrap();
        let reading_start = reading_text
            .find("\"id\":\"saved_")
            .expect("saved reading problem id");
        let reading_value = &reading_text[reading_start + 6..];
        let reading_end = reading_value.find('"').expect("saved id end quote");
        let reading_id = &reading_value[..reading_end];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{speaking_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":777777777,"append_note":"speaking collapsed"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{reading_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":888888888,"append_note":"reading clause dropped"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/weakness-queue?source=reviewed&limit=2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"mode\":\"speaking\""));
        assert!(text.contains("\"mode\":\"reading\""));
        assert!(text.contains(speaking_id));
        assert!(text.contains(reading_id));
    }

    #[tokio::test]
    async fn problem_bank_dashboard_combines_problem_bank_views() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":999999999,"append_note":"dashboard retry"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/dashboard?preferred_mode=speaking&activity_source=reviewed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 32768).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"stats\""));
        assert!(text.contains("\"insights\""));
        assert!(text.contains("\"review_queue\""));
        assert!(text.contains("\"weakness_queue\""));
        assert!(text.contains("\"recommended_next_mode\":\"speaking\""));
        assert!(text.contains("\"stale_problems\""));
        assert!(text.contains("\"failure_rate_by_mode\""));
        assert!(text.contains("\"focus_tag_bias\""));
        assert!(text.contains("\"mode_summary\""));
        assert!(text.contains("\"trend\""));
        assert!(text.contains("\"risk_level\""));
        assert!(text.contains("\"next_action\""));
        assert!(text.contains("\"alerts\""));
        assert!(text.contains(saved_id));
    }

    #[tokio::test]
    async fn problem_bank_stale_returns_idle_problems() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":true,"occurred_at_unix":1,"append_note":"very old usage"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/stale?source=reviewed&stale_after_days=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"total\":1"));
        assert!(text.contains(saved_id));
        assert!(text.contains("\"idle_days\""));
    }

    #[tokio::test]
    async fn problem_bank_snapshots_capture_and_list() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":1234567890,"append_note":"snapshot route"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let capture_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/snapshots/capture?preferred_mode=speaking&activity_source=reviewed")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"note":"route snapshot"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(capture_response.status(), StatusCode::CREATED);
        let capture_body = to_bytes(capture_response.into_body(), 32768).await.unwrap();
        let capture_text = String::from_utf8(capture_body.to_vec()).unwrap();
        assert!(capture_text.contains("\"note\":\"route snapshot\""));
        assert!(capture_text.contains("\"dashboard\""));

        let list_response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/snapshots?limit=5")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(list_response.status(), StatusCode::OK);
        let list_body = to_bytes(list_response.into_body(), 32768).await.unwrap();
        let list_text = String::from_utf8(list_body.to_vec()).unwrap();
        assert!(list_text.contains("\"total\":1"));
        assert!(list_text.contains("\"route snapshot\""));
    }

    #[tokio::test]
    async fn problem_bank_save_persists_generated_items_in_memory() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/save")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "source":"reviewed",
                            "generated_set":{
                                "source_text":"The client approved the design draft, but the delivery schedule is still under review.",
                                "summary":"The client approved the design draft",
                                "target_context":"meeting",
                                "level_band":"toeic_750_800",
                                "topic":"meeting",
                                "items":[
                                    {
                                        "id":"gen_read",
                                        "title":"Generated Decision Lock",
                                        "mode":"reading",
                                        "level_band":"toeic_750_800",
                                        "topic":"meeting",
                                        "target_context":"meeting",
                                        "prompt":"Read the decision first.",
                                        "wm_support":"Keep the decision stable.",
                                        "success_check":"You can say the decision.",
                                        "tags":["generated","core_lock"],
                                        "sort_order":10
                                    }
                                ]
                            }
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"saved_count\":1"));
        assert!(text.contains("\"source\":\"reviewed\""));
    }

    #[tokio::test]
    async fn problem_bank_delete_removes_saved_item() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/save")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "source":"generated",
                            "generated_set":{
                                "source_text":"The client approved the design draft, but the delivery schedule is still under review.",
                                "summary":"The client approved the design draft",
                                "target_context":"meeting",
                                "level_band":"toeic_750_800",
                                "topic":"meeting",
                                "items":[
                                    {
                                        "id":"gen_delete",
                                        "title":"Generated Decision Lock",
                                        "mode":"reading",
                                        "level_band":"toeic_750_800",
                                        "topic":"meeting",
                                        "target_context":"meeting",
                                        "prompt":"Read the decision first.",
                                        "wm_support":"Keep the decision stable.",
                                        "success_check":"You can say the decision.",
                                        "tags":["generated","core_lock"],
                                        "sort_order":10
                                    }
                                ]
                            }
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(save_response.status(), StatusCode::CREATED);
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let delete_response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/problem-bank/{saved_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::OK);
        let delete_body = to_bytes(delete_response.into_body(), 4096).await.unwrap();
        let delete_text = String::from_utf8(delete_body.to_vec()).unwrap();
        assert!(delete_text.contains(saved_id));
    }

    #[tokio::test]
    async fn problem_bank_patch_updates_saved_item() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/save")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "source":"reviewed",
                            "generated_set":{
                                "source_text":"The client approved the design draft, but the delivery schedule is still under review.",
                                "summary":"The client approved the design draft",
                                "target_context":"meeting",
                                "level_band":"toeic_750_800",
                                "topic":"meeting",
                                "items":[
                                    {
                                        "id":"gen_patch",
                                        "title":"Generated Decision Lock",
                                        "mode":"reading",
                                        "level_band":"toeic_750_800",
                                        "topic":"meeting",
                                        "target_context":"meeting",
                                        "prompt":"Read the decision first.",
                                        "wm_support":"Keep the decision stable.",
                                        "success_check":"You can say the decision.",
                                        "tags":["generated","core_lock"],
                                        "sort_order":10
                                    }
                                ]
                            }
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let patch_response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/problem-bank/{saved_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"title":"Pinned meeting prompt","notes":"good for rehearsal","pinned":true}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(patch_response.status(), StatusCode::OK);
        let patch_body = to_bytes(patch_response.into_body(), 4096).await.unwrap();
        let patch_text = String::from_utf8(patch_body.to_vec()).unwrap();
        assert!(patch_text.contains("Pinned meeting prompt"));
        assert!(patch_text.contains("\"pinned\":true"));
    }

    #[tokio::test]
    async fn problem_bank_clone_saves_seed_problem() {
        let app = build_router(state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"saved_count\":1"));
        assert!(text.contains("Two-Step Link: Status Update"));
    }

    #[tokio::test]
    async fn problem_bank_custom_lists_saved_items_only() {
        let app = build_router(state());

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/custom")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"saved_"));
        assert!(text.contains("\"source\":\"reviewed\""));
    }

    #[tokio::test]
    async fn problem_bank_custom_supports_pinned_source_filters() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/problem-bank/{saved_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"pinned":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/custom?source=reviewed&pinned_only=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"pinned\":true"));
        assert!(text.contains("\"source\":\"reviewed\""));
    }

    #[tokio::test]
    async fn problem_bank_usage_updates_saved_item() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/save")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{
                            "source":"generated",
                            "generated_set":{
                                "source_text":"The study found lower overload, but live conversation data is still limited.",
                                "summary":"The study found lower overload",
                                "target_context":"research",
                                "level_band":"toeic_750_800",
                                "topic":"research",
                                "items":[
                                    {
                                        "id":"gen_usage",
                                        "title":"Generated Research Core Lock",
                                        "mode":"reading",
                                        "level_band":"toeic_750_800",
                                        "topic":"research",
                                        "target_context":"research",
                                        "prompt":"Read the claim first.",
                                        "wm_support":"Hold the claim first.",
                                        "success_check":"You can say the claim.",
                                        "tags":["generated","core_lock"],
                                        "sort_order":10
                                    }
                                ]
                            }
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let usage_response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":true,"occurred_at_unix":123456789,"append_note":"worked in a short session"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(usage_response.status(), StatusCode::OK);
        let usage_body = to_bytes(usage_response.into_body(), 4096).await.unwrap();
        let usage_text = String::from_utf8(usage_body.to_vec()).unwrap();
        assert!(usage_text.contains("\"usage_count\":1"));
        assert!(usage_text.contains("\"success_count\":1"));
        assert!(usage_text.contains("\"last_used_unix\":123456789"));
    }

    #[tokio::test]
    async fn problem_bank_history_returns_usage_entries() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":true,"occurred_at_unix":222222222,"append_note":"stable on second try"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let history_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/problem-bank/{saved_id}/history"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(history_response.status(), StatusCode::OK);
        let history_body = to_bytes(history_response.into_body(), 4096).await.unwrap();
        let history_text = String::from_utf8(history_body.to_vec()).unwrap();
        assert!(history_text.contains("\"total\":1"));
        assert!(history_text.contains("\"successful\":true"));
        assert!(history_text.contains("\"occurred_at_unix\":222222222"));
    }

    #[tokio::test]
    async fn problem_bank_activity_returns_recent_usage_feed() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/problem-bank/{saved_id}"))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"pinned":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":true,"occurred_at_unix":333333333,"append_note":"worked well in rehearsal"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/activity?source=reviewed&pinned_only=true&successful=true")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"total\":1"));
        assert!(text.contains("\"successful\":true"));
        assert!(text.contains("\"occurred_at_unix\":333333333"));
        assert!(text.contains("worked well in rehearsal"));
    }

    #[tokio::test]
    async fn problem_bank_insights_summarize_usage_patterns() {
        let app = build_router(state());

        let save_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/problem-bank/pb_speak_002/save")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"source":"reviewed"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let save_body = to_bytes(save_response.into_body(), 4096).await.unwrap();
        let save_text = String::from_utf8(save_body.to_vec()).unwrap();
        let id_start = save_text.find("\"id\":\"saved_").expect("saved problem id");
        let id_value = &save_text[id_start + 6..];
        let end_quote = id_value.find('"').expect("saved id end quote");
        let saved_id = &id_value[..end_quote];

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":true,"occurred_at_unix":444444444,"append_note":"clear on first try"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/problem-bank/{saved_id}/usage"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"successful":false,"occurred_at_unix":555555555,"append_note":"lost the second step"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/problem-bank/insights?source=reviewed")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 8192).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("\"total_history_entries\":2"));
        assert!(text.contains("\"successful_history_entries\":1"));
        assert!(text.contains("\"failed_history_entries\":1"));
        assert!(text.contains("\"overall_success_rate\":0.5"));
        assert!(text.contains(saved_id));
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
                problem_bank_path: None,
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

    async fn spawn_problem_worker_server() -> TestServer {
        let app = AxumRouter::new()
            .route(
                "/analyze/reader-plan",
                axum::routing::post(|| async {
                    (
                        StatusCode::OK,
                        AxumJson(serde_json::json!({
                            "summary": "The client approved the design draft.",
                            "focus_steps": [
                                {
                                    "text": "The client approved the design draft",
                                    "guidance_en": "Keep the decision stable before adding schedule detail."
                                }
                            ],
                            "hotspots": [
                                {
                                    "recommendation": "Hide the schedule clause until the decision is clear."
                                }
                            ]
                        })),
                    )
                }),
            )
            .route(
                "/analyze/listening-plan",
                axum::routing::post(|| async {
                    (
                        StatusCode::OK,
                        AxumJson(serde_json::json!({
                            "recommended_speed": "0.85x",
                            "pause_points": [
                                {
                                    "after_chunk_order": 1,
                                    "cue_en": "Say the decision first.",
                                    "preview_text": "The client approved the design draft"
                                }
                            ],
                            "final_pass_strategy": "Listen once for the decision, then replay for the schedule detail."
                        })),
                    )
                }),
            )
            .route(
                "/analyze/speaking-plan",
                axum::routing::post(|| async {
                    (
                        StatusCode::OK,
                        AxumJson(serde_json::json!({
                            "summary": "The client approved the design draft.",
                            "recommended_style": "two_short_steps",
                            "opener_options": ["The decision is approved."],
                            "steps": [
                                {
                                    "text": "The client approved the design draft.",
                                    "delivery_tip_en": "Finish the decision sentence before the schedule detail."
                                },
                                {
                                    "text": "The delivery schedule is still under review.",
                                    "delivery_tip_en": "Keep the follow-up detail in a second short sentence."
                                }
                            ]
                        })),
                    )
                }),
            )
            .route(
                "/analyze/rescue-plan",
                axum::routing::post(|| async {
                    (
                        StatusCode::OK,
                        AxumJson(serde_json::json!({
                            "overload_level": "medium",
                            "primary_strategy": "decision_first",
                            "phrases": [
                                {
                                    "phrase_en": "Could you give me the decision first?",
                                    "use_when": "you hear the detail before the main meeting point"
                                }
                            ]
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
