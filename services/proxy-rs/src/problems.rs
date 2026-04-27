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
        GeneratedProblemSet, ProblemFilter, ProblemGenerationRequest, ProblemRecord,
        ProblemSaveSource,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveGeneratedProblemRequest {
    generated_set: GeneratedProblemSet,
    source: ProblemSaveSource,
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
