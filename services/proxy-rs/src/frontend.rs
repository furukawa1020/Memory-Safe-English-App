use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::{
    config::RuntimeEnvironment,
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
            Json(MobileBootstrapResponse::from_state(&state, &readiness)),
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
    recommended_base_urls: Option<RecommendedBaseUrls>,
    routes: FrontendRoutes,
    capabilities: FrontendCapabilities,
    api: Option<readiness::UpstreamStatus>,
    worker: Option<readiness::UpstreamStatus>,
    environment: &'static str,
}

impl MobileBootstrapResponse {
    fn from_state(state: &AppState, report: &readiness::ReadinessReport) -> Self {
        let is_production = state.config.runtime_environment == RuntimeEnvironment::Production;
        Self {
            ready: report.ready,
            checked_at_unix_ms: report.checked_at_unix_ms,
            recommended_base_urls: if is_production {
                None
            } else {
                Some(RecommendedBaseUrls::default())
            },
            routes: FrontendRoutes::default(),
            capabilities: FrontendCapabilities::default(),
            api: if is_production {
                None
            } else {
                Some(report.api.clone())
            },
            worker: if is_production {
                None
            } else {
                Some(report.worker.clone())
            },
            environment: if is_production { "production" } else { "development" },
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
    skeleton_analysis: &'static str,
    reader_plan: &'static str,
    listening_plan: &'static str,
    speaking_plan: &'static str,
    rescue_plan: &'static str,
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
            skeleton_analysis: "/analysis/skeleton",
            reader_plan: "/analysis/reader-plan",
            listening_plan: "/analysis/listening-plan",
            speaking_plan: "/analysis/speaking-plan",
            rescue_plan: "/analysis/rescue-plan",
        }
    }
}

#[derive(Debug, Serialize)]
struct FrontendCapabilities {
    chunk_reader: bool,
    skeleton_reader: bool,
    reader_plan: bool,
    listening_plan: bool,
    speaking_plan: bool,
    rescue_plan: bool,
    onboarding_assessment: bool,
    analytics_summary: bool,
}

impl Default for FrontendCapabilities {
    fn default() -> Self {
        Self {
            chunk_reader: true,
            skeleton_reader: true,
            reader_plan: true,
            listening_plan: true,
            speaking_plan: true,
            rescue_plan: true,
            onboarding_assessment: true,
            analytics_summary: true,
        }
    }
}
