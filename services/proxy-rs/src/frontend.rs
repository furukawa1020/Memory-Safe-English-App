use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::{
    http_response::with_standard_headers, readiness, request_id::resolve_request_id,
    state::AppState,
};

pub async fn mobile_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let readiness = readiness::probe_upstreams(&state).await;

    with_standard_headers(
        (
            StatusCode::OK,
            Json(MobileBootstrapResponse::from_readiness(&readiness)),
        )
            .into_response(),
        &request_id,
        "miss",
    )
}

#[derive(Debug, Serialize)]
struct MobileBootstrapResponse {
    ready: bool,
    checked_at_unix_ms: u128,
    recommended_base_urls: RecommendedBaseUrls,
    routes: FrontendRoutes,
    api: readiness::UpstreamStatus,
    worker: readiness::UpstreamStatus,
}

impl MobileBootstrapResponse {
    fn from_readiness(report: &readiness::ReadinessReport) -> Self {
        Self {
            ready: report.ready,
            checked_at_unix_ms: report.checked_at_unix_ms,
            recommended_base_urls: RecommendedBaseUrls::default(),
            routes: FrontendRoutes::default(),
            api: report.api.clone(),
            worker: report.worker.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct RecommendedBaseUrls {
    android_emulator: &'static str,
    ios_simulator: &'static str,
    desktop: &'static str,
}

impl Default for RecommendedBaseUrls {
    fn default() -> Self {
        Self {
            android_emulator: "http://10.0.2.2:8070",
            ios_simulator: "http://127.0.0.1:8070",
            desktop: "http://127.0.0.1:8070",
        }
    }
}

#[derive(Debug, Serialize)]
struct FrontendRoutes {
    readiness: &'static str,
    login: &'static str,
    register: &'static str,
    refresh: &'static str,
    current_user: &'static str,
    contents: &'static str,
    chunk_analysis: &'static str,
}

impl Default for FrontendRoutes {
    fn default() -> Self {
        Self {
            readiness: "/ready",
            login: "/auth/login",
            register: "/auth/register",
            refresh: "/auth/refresh",
            current_user: "/me",
            contents: "/contents",
            chunk_analysis: "/analysis/chunks",
        }
    }
}
