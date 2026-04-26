use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    http_response::with_standard_headers,
    problem_bank::{GeneratedProblemSet, ProblemFilter, ProblemGenerationRequest, ProblemRecord},
    request_id::resolve_request_id,
    response_headers::HeaderPolicy,
    state::AppState,
};

pub async fn list_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemBankQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let items = state.problem_bank.list(ProblemFilter {
        mode: query.mode,
        level_band: query.level_band,
        topic: query.topic,
        target_context: query.target_context,
        query: query.query,
        limit,
    });

    with_standard_headers(
        (
            StatusCode::OK,
            Json(ProblemBankListResponse {
                total: items.len(),
                items,
            }),
        )
            .into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

pub async fn get_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.get(&id) {
        Some(item) => with_standard_headers(
            (StatusCode::OK, Json(item)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        None => with_standard_headers(
            (
                StatusCode::NOT_FOUND,
                Json(ProblemBankErrorResponse {
                    error: "problem_not_found",
                }),
            )
                .into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
    }
}

pub async fn generate_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ProblemGenerationRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let generated = state.problem_bank.generate(request);

    with_standard_headers(
        (StatusCode::OK, Json(GeneratedProblemSetResponse::from(generated))).into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

#[derive(Debug, Deserialize)]
pub struct ProblemBankQuery {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ProblemBankListResponse {
    total: usize,
    items: Vec<ProblemRecord>,
}

#[derive(Debug, Serialize)]
struct GeneratedProblemSetResponse {
    source_text: String,
    summary: String,
    target_context: String,
    level_band: String,
    topic: String,
    items: Vec<ProblemRecord>,
}

impl From<GeneratedProblemSet> for GeneratedProblemSetResponse {
    fn from(value: GeneratedProblemSet) -> Self {
        Self {
            source_text: value.source_text,
            summary: value.summary,
            target_context: value.target_context,
            level_band: value.level_band,
            topic: value.topic,
            items: value.items,
        }
    }
}

#[derive(Debug, Serialize)]
struct ProblemBankErrorResponse {
    error: &'static str,
}
