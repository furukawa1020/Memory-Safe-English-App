use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::join;

use crate::{
    http_response::with_standard_headers,
    problem_bank::{
        GeneratedProblemSet, ProblemActivityEntry, ProblemActivityRequest, ProblemFilter,
        ProblemGenerationRequest, ProblemRecommendationRequest, ProblemRecord,
        ProblemRecordUpdate, ProblemSaveSource, ProblemStaleRequest, ProblemUsageEvent,
        ProblemUsageHistory,
    },
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
        source: query.source,
        tag: query.tag,
        pinned_only: query.pinned_only.unwrap_or(false),
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

pub async fn list_custom_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemBankQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let items = state.problem_bank.list_custom(ProblemFilter {
        mode: query.mode,
        level_band: query.level_band,
        topic: query.topic,
        target_context: query.target_context,
        source: query.source,
        tag: query.tag,
        pinned_only: query.pinned_only.unwrap_or(false),
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

pub async fn problem_activity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemActivityQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let items = state.problem_bank.activity(ProblemActivityRequest {
        mode: query.mode,
        level_band: query.level_band,
        topic: query.topic,
        target_context: query.target_context,
        source: query.source,
        query: query.query,
        successful: query.successful,
        pinned_only: query.pinned_only.unwrap_or(false),
        limit: query.limit.unwrap_or(20).clamp(1, 100),
    });

    with_standard_headers(
        (
            StatusCode::OK,
            Json(ProblemActivityResponse {
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

pub async fn problem_insights(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemActivityQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let insights = state.problem_bank.insights(ProblemActivityRequest {
        mode: query.mode,
        level_band: query.level_band,
        topic: query.topic,
        target_context: query.target_context,
        source: query.source,
        query: query.query,
        successful: query.successful,
        pinned_only: query.pinned_only.unwrap_or(false),
        limit: query.limit.unwrap_or(10).clamp(1, 100),
    });

    with_standard_headers(
        (StatusCode::OK, Json(insights)).into_response(),
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

pub async fn problem_history(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.history(&id) {
        Some(history) => with_standard_headers(
            (
                StatusCode::OK,
                Json(ProblemHistoryResponse {
                    total: history.len(),
                    history,
                }),
            )
                .into_response(),
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

pub async fn delete_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.delete_custom(&id) {
        Ok(deleted) => with_standard_headers(
            (StatusCode::OK, Json(deleted)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        Err(crate::problem_bank::ProblemBankDeleteError::NotFound) => with_standard_headers(
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
        Err(crate::problem_bank::ProblemBankDeleteError::Persist(_)) => with_standard_headers(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemBankErrorResponse {
                    error: "problem_bank_delete_failed",
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

pub async fn clone_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<CloneProblemRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.clone_problem(&id, request.source) {
        Ok(saved) => with_standard_headers(
            (StatusCode::CREATED, Json(saved)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        Err(crate::problem_bank::ProblemBankSaveError::ProblemNotFound) => with_standard_headers(
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
        Err(_) => with_standard_headers(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemBankErrorResponse {
                    error: "problem_bank_clone_failed",
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

pub async fn update_problem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<ProblemUpdateRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.update_custom(&id, request.into()) {
        Ok(updated) => with_standard_headers(
            (StatusCode::OK, Json(updated)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        Err(crate::problem_bank::ProblemBankUpdateError::NotFound) => with_standard_headers(
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
        Err(crate::problem_bank::ProblemBankUpdateError::Persist(_)) => with_standard_headers(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemBankErrorResponse {
                    error: "problem_bank_update_failed",
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

pub async fn record_problem_usage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(request): Json<ProblemUsageRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state.problem_bank.record_usage(&id, request.into()) {
        Ok(updated) => with_standard_headers(
            (StatusCode::OK, Json(updated)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        Err(crate::problem_bank::ProblemBankUpdateError::NotFound) => with_standard_headers(
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
        Err(crate::problem_bank::ProblemBankUpdateError::Persist(_)) => with_standard_headers(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemBankErrorResponse {
                    error: "problem_bank_usage_failed",
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
    let generated = enrich_generated_problem_set(&state, request).await;

    with_standard_headers(
        (StatusCode::OK, Json(GeneratedProblemSetResponse::from(generated))).into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

pub async fn problem_bank_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let stats = state.problem_bank.stats();

    with_standard_headers(
        (StatusCode::OK, Json(stats)).into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

pub async fn recommend_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemRecommendationQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let items = state.problem_bank.recommend(ProblemRecommendationRequest {
        preferred_mode: query.preferred_mode,
        target_context: query.target_context,
        level_band: query.level_band,
        topic: query.topic,
        focus_tag: query.focus_tag,
        prefer_review: query.prefer_review.unwrap_or(false),
        avoid_mastered: query.avoid_mastered.unwrap_or(false),
        limit: query.limit.unwrap_or(5),
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

pub async fn review_queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemRecommendationQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let items = state.problem_bank.review_queue(ProblemRecommendationRequest {
        preferred_mode: query.preferred_mode,
        target_context: query.target_context,
        level_band: query.level_band,
        topic: query.topic,
        focus_tag: query.focus_tag,
        prefer_review: query.prefer_review.unwrap_or(true),
        avoid_mastered: query.avoid_mastered.unwrap_or(true),
        limit: query.limit.unwrap_or(5),
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

pub async fn weakness_queue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemRecommendationQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let queue = state.problem_bank.weakness_queue(ProblemRecommendationRequest {
        preferred_mode: query.preferred_mode,
        target_context: query.target_context,
        level_band: query.level_band,
        topic: query.topic,
        focus_tag: query.focus_tag,
        prefer_review: query.prefer_review.unwrap_or(true),
        avoid_mastered: query.avoid_mastered.unwrap_or(true),
        limit: query.limit.unwrap_or(3),
    });

    with_standard_headers(
        (StatusCode::OK, Json(queue)).into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

pub async fn problem_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemDashboardQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let dashboard = state.problem_bank.dashboard(
        ProblemRecommendationRequest {
            preferred_mode: query.preferred_mode.clone(),
            target_context: query.target_context.clone(),
            level_band: query.level_band.clone(),
            topic: query.topic.clone(),
            focus_tag: query.focus_tag.clone(),
            prefer_review: query.prefer_review.unwrap_or(true),
            avoid_mastered: query.avoid_mastered.unwrap_or(true),
            limit: query.limit.unwrap_or(5),
        },
        ProblemActivityRequest {
            mode: query.activity_mode,
            level_band: query.activity_level_band,
            topic: query.activity_topic,
            target_context: query.activity_target_context,
            source: query.activity_source,
            query: query.activity_query,
            successful: query.activity_successful,
            pinned_only: query.activity_pinned_only.unwrap_or(false),
            limit: query.activity_limit.unwrap_or(10).clamp(1, 100),
        },
        ProblemStaleRequest {
            mode: query.stale_mode,
            target_context: query.stale_target_context,
            source: query.stale_source,
            pinned_only: query.stale_pinned_only.unwrap_or(false),
            stale_after_days: query.stale_after_days.unwrap_or(7),
            limit: query.stale_limit.unwrap_or(10).clamp(1, 100),
        },
    );

    with_standard_headers(
        (StatusCode::OK, Json(dashboard)).into_response(),
        &request_id,
        "miss",
        &state.config.runtime_environment,
        HeaderPolicy::Sensitive,
    )
}

pub async fn stale_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ProblemStaleQuery>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    let items = state.problem_bank.stale_problems(ProblemStaleRequest {
        mode: query.mode,
        target_context: query.target_context,
        source: query.source,
        pinned_only: query.pinned_only.unwrap_or(false),
        stale_after_days: query.stale_after_days.unwrap_or(7),
        limit: query.limit.unwrap_or(10).clamp(1, 100),
    });

    with_standard_headers(
        (
            StatusCode::OK,
            Json(StaleProblemResponse {
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

pub async fn save_generated_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SaveGeneratedProblemRequest>,
) -> impl IntoResponse {
    let request_id = resolve_request_id(&headers);
    match state
        .problem_bank
        .save_generated_set(&request.generated_set, request.source)
    {
        Ok(saved) => with_standard_headers(
            (StatusCode::CREATED, Json(saved)).into_response(),
            &request_id,
            "miss",
            &state.config.runtime_environment,
            HeaderPolicy::Sensitive,
        ),
        Err(_) => with_standard_headers(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProblemBankErrorResponse {
                    error: "problem_bank_save_failed",
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

#[derive(Debug, Deserialize)]
pub struct ProblemBankQuery {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub pinned_only: Option<bool>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemActivityQuery {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub pinned_only: Option<bool>,
    pub successful: Option<bool>,
    pub query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemRecommendationQuery {
    pub preferred_mode: Option<String>,
    pub target_context: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub focus_tag: Option<String>,
    pub prefer_review: Option<bool>,
    pub avoid_mastered: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemDashboardQuery {
    pub preferred_mode: Option<String>,
    pub target_context: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub focus_tag: Option<String>,
    pub prefer_review: Option<bool>,
    pub avoid_mastered: Option<bool>,
    pub limit: Option<usize>,
    pub activity_mode: Option<String>,
    pub activity_level_band: Option<String>,
    pub activity_topic: Option<String>,
    pub activity_target_context: Option<String>,
    pub activity_source: Option<String>,
    pub activity_query: Option<String>,
    pub activity_successful: Option<bool>,
    pub activity_pinned_only: Option<bool>,
    pub activity_limit: Option<usize>,
    pub stale_mode: Option<String>,
    pub stale_target_context: Option<String>,
    pub stale_source: Option<String>,
    pub stale_pinned_only: Option<bool>,
    pub stale_after_days: Option<u64>,
    pub stale_limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemStaleQuery {
    pub mode: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub pinned_only: Option<bool>,
    pub stale_after_days: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ProblemBankListResponse {
    total: usize,
    items: Vec<ProblemRecord>,
}

#[derive(Debug, Serialize)]
struct ProblemHistoryResponse {
    total: usize,
    history: Vec<ProblemUsageHistory>,
}

#[derive(Debug, Serialize)]
struct ProblemActivityResponse {
    total: usize,
    items: Vec<ProblemActivityEntry>,
}

#[derive(Debug, Serialize)]
struct StaleProblemResponse {
    total: usize,
    items: Vec<crate::problem_bank::ProblemStaleEntry>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveGeneratedProblemRequest {
    generated_set: GeneratedProblemSet,
    source: ProblemSaveSource,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloneProblemRequest {
    source: ProblemSaveSource,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemUpdateRequest {
    title: Option<String>,
    prompt: Option<String>,
    wm_support: Option<String>,
    success_check: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    pinned: Option<bool>,
}

impl From<ProblemUpdateRequest> for ProblemRecordUpdate {
    fn from(value: ProblemUpdateRequest) -> Self {
        Self {
            title: value.title,
            prompt: value.prompt,
            wm_support: value.wm_support,
            success_check: value.success_check,
            tags: value.tags,
            notes: value.notes,
            pinned: value.pinned,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemUsageRequest {
    successful: bool,
    occurred_at_unix: Option<u64>,
    append_note: Option<String>,
}

impl From<ProblemUsageRequest> for ProblemUsageEvent {
    fn from(value: ProblemUsageRequest) -> Self {
        Self {
            successful: value.successful,
            occurred_at_unix: value.occurred_at_unix,
            append_note: value.append_note,
        }
    }
}

#[derive(Debug, Serialize)]
struct ProblemBankErrorResponse {
    error: &'static str,
}

async fn enrich_generated_problem_set(
    state: &AppState,
    request: ProblemGenerationRequest,
) -> GeneratedProblemSet {
    let mut generated = state.problem_bank.generate(request.clone());
    let target_context = generated.target_context.clone();
    let plan_request = WorkerPlanRequest {
        text: generated.source_text.clone(),
        target_context,
        language: "en",
    };

    let (reader, listening, speaking, rescue) = join!(
        fetch_worker_plan::<ReaderPlanResponse>(
            &state.http_client,
            &state.config.worker_base_url,
            "/analyze/reader-plan",
            &plan_request,
        ),
        fetch_worker_plan::<ListeningPlanResponse>(
            &state.http_client,
            &state.config.worker_base_url,
            "/analyze/listening-plan",
            &plan_request,
        ),
        fetch_worker_plan::<SpeakingPlanResponse>(
            &state.http_client,
            &state.config.worker_base_url,
            "/analyze/speaking-plan",
            &plan_request,
        ),
        fetch_worker_plan::<RescuePlanResponse>(
            &state.http_client,
            &state.config.worker_base_url,
            "/analyze/rescue-plan",
            &plan_request,
        ),
    );

    if let Some(reader) = reader {
        generated.summary = non_empty_or(reader.summary, generated.summary);
        if let Some(item) = generated.items.iter_mut().find(|item| item.mode == "reading") {
            if let Some(step) = reader.focus_steps.first() {
                item.prompt = format!(
                    "{} Focus here first: '{}'. {}",
                    item.prompt,
                    step.text,
                    step.guidance_en
                );
            }
            if let Some(hotspot) = reader.hotspots.first() {
                item.wm_support = format!(
                    "{} Hotspot: {}",
                    item.wm_support,
                    hotspot.recommendation
                );
            }
        }
    }

    if let Some(listening) = listening {
        if let Some(item) = generated.items.iter_mut().find(|item| item.mode == "listening") {
            if let Some(point) = listening.pause_points.first() {
                item.prompt = format!(
                    "Pause after chunk {} and focus on '{}'. Cue: {}",
                    point.after_chunk_order, point.preview_text, point.cue_en
                );
                item.success_check = format!(
                    "You can say the checkpoint meaning before the next chunk at {} speed.",
                    listening.recommended_speed
                );
            }
            item.wm_support = format!(
                "{} Final pass: {}",
                item.wm_support, listening.final_pass_strategy
            );
        }
    }

    if let Some(speaking) = speaking {
        generated.summary = non_empty_or(speaking.summary.clone(), generated.summary);
        if let Some(item) = generated.items.iter_mut().find(|item| item.mode == "speaking") {
            let opener = speaking
                .opener_options
                .first()
                .cloned()
                .unwrap_or_else(|| generated.summary.clone());
            let steps = speaking
                .steps
                .iter()
                .take(2)
                .map(|step| step.text.as_str())
                .collect::<Vec<_>>()
                .join(" Then say: ");
            item.prompt = if steps.is_empty() {
                format!("Start with: '{}'.", opener)
            } else {
                format!("Start with: '{}'. Then say: {}", opener, steps)
            };
            item.wm_support = format!(
                "{} Preferred style: {}.",
                item.wm_support, speaking.recommended_style
            );
            if let Some(step) = speaking.steps.first() {
                item.success_check = format!(
                    "You can deliver the opener and first short step. Tip: {}",
                    step.delivery_tip_en
                );
            }
        }
    }

    if let Some(rescue) = rescue {
        if let Some(item) = generated.items.iter_mut().find(|item| item.mode == "rescue") {
            if let Some(phrase) = rescue.phrases.first() {
                item.prompt = format!("Practice saying: '{}'", phrase.phrase_en);
                item.success_check = format!(
                    "You can use it when {}.",
                    phrase.use_when.to_ascii_lowercase()
                );
            }
            item.wm_support = format!(
                "{} Strategy: {} (overload: {}).",
                item.wm_support, rescue.primary_strategy, rescue.overload_level
            );
        }
    }

    generated
}

async fn fetch_worker_plan<T: DeserializeOwned>(
    http_client: &reqwest::Client,
    worker_base_url: &str,
    path: &str,
    request: &WorkerPlanRequest,
) -> Option<T> {
    let url = format!("{worker_base_url}{path}");
    let response = http_client.post(url).json(request).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json::<T>().await.ok()
}

fn non_empty_or(candidate: String, fallback: String) -> String {
    if candidate.trim().is_empty() {
        fallback
    } else {
        candidate
    }
}

#[derive(Debug, Serialize)]
struct WorkerPlanRequest {
    text: String,
    target_context: String,
    language: &'static str,
}

#[derive(Debug, Deserialize)]
struct ReaderPlanResponse {
    summary: String,
    focus_steps: Vec<ReaderFocusStep>,
    hotspots: Vec<ReaderHotspot>,
}

#[derive(Debug, Deserialize)]
struct ReaderFocusStep {
    text: String,
    guidance_en: String,
}

#[derive(Debug, Deserialize)]
struct ReaderHotspot {
    recommendation: String,
}

#[derive(Debug, Deserialize)]
struct ListeningPlanResponse {
    recommended_speed: String,
    pause_points: Vec<ListeningPausePoint>,
    final_pass_strategy: String,
}

#[derive(Debug, Deserialize)]
struct ListeningPausePoint {
    after_chunk_order: i32,
    cue_en: String,
    preview_text: String,
}

#[derive(Debug, Deserialize)]
struct SpeakingPlanResponse {
    summary: String,
    recommended_style: String,
    opener_options: Vec<String>,
    steps: Vec<SpeakingStep>,
}

#[derive(Debug, Deserialize)]
struct SpeakingStep {
    text: String,
    delivery_tip_en: String,
}

#[derive(Debug, Deserialize)]
struct RescuePlanResponse {
    overload_level: String,
    primary_strategy: String,
    phrases: Vec<RescuePhrase>,
}

#[derive(Debug, Deserialize)]
struct RescuePhrase {
    phrase_en: String,
    use_when: String,
}
