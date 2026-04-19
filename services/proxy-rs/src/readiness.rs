use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use tokio::join;

use crate::{
    http_response::with_standard_headers, request_id::resolve_request_id, state::AppState,
};

pub async fn ready(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let report = probe_upstreams(&state).await;
    let status = if report.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    with_standard_headers((status, Json(report)).into_response(), &request_id, "miss")
}

pub async fn probe_upstreams(state: &AppState) -> ReadinessReport {
    let api_url = health_url(&state.config.api_base_url);
    let worker_url = health_url(&state.config.worker_base_url);

    let (api, worker) = join!(
        probe_target(&state.http_client, "api", api_url),
        probe_target(&state.http_client, "worker", worker_url)
    );

    ReadinessReport {
        ready: api.ok && worker.ok,
        checked_at_unix_ms: unix_time_millis(),
        api,
        worker,
    }
}

async fn probe_target(client: &reqwest::Client, name: &'static str, url: String) -> UpstreamStatus {
    match client.get(&url).send().await {
        Ok(response) => UpstreamStatus {
            name,
            ok: response.status().is_success(),
            status_code: response.status().as_u16(),
            url,
        },
        Err(_) => UpstreamStatus {
            name,
            ok: false,
            status_code: 0,
            url,
        },
    }
}

fn health_url(base_url: &str) -> String {
    format!("{base_url}/health")
}

fn unix_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[derive(Debug, Serialize)]
pub struct ReadinessReport {
    pub ready: bool,
    pub checked_at_unix_ms: u128,
    pub api: UpstreamStatus,
    pub worker: UpstreamStatus,
}

#[derive(Debug, Serialize)]
pub struct UpstreamStatus {
    pub name: &'static str,
    pub ok: bool,
    pub status_code: u16,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_url_appends_health_path() {
        assert_eq!(health_url("http://127.0.0.1:8080"), "http://127.0.0.1:8080/health");
    }
}
