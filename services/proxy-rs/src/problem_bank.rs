use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct ProblemBank {
    store: Arc<RwLock<ProblemStore>>,
    persisted_path: Option<PathBuf>,
}

impl ProblemBank {
    pub fn seeded() -> Self {
        Self::new(None)
    }

    pub fn with_persisted_path(path: impl Into<PathBuf>) -> Self {
        Self::new(Some(path.into()))
    }

    fn new(persisted_path: Option<PathBuf>) -> Self {
        let seeded = seeded_records();
        let custom = persisted_path
            .as_ref()
            .and_then(|path| load_custom_records(path).ok())
            .unwrap_or_default();
        let store = ProblemStore::new(seeded, custom);

        Self {
            store: Arc::new(RwLock::new(store)),
            persisted_path,
        }
    }

    pub fn list(&self, filter: ProblemFilter) -> Vec<ProblemRecord> {
        let store = self.store.read().expect("problem bank read lock");
        let mut matched = store
            .all_items()
            .into_iter()
            .filter(|item| filter.matches(item))
            .collect::<Vec<_>>();
        matched.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then_with(|| a.id.cmp(&b.id)));
        if matched.len() > filter.limit {
            matched.truncate(filter.limit);
        }
        matched
    }

    pub fn get(&self, id: &str) -> Option<ProblemRecord> {
        let store = self.store.read().expect("problem bank read lock");
        store.by_id.get(id).cloned()
    }

    pub fn list_custom(&self, filter: ProblemFilter) -> Vec<ProblemRecord> {
        let store = self.store.read().expect("problem bank read lock");
        let mut matched = store
            .custom
            .values()
            .filter(|item| filter.matches(item))
            .cloned()
            .collect::<Vec<_>>();
        matched.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then_with(|| a.id.cmp(&b.id)));
        if matched.len() > filter.limit {
            matched.truncate(filter.limit);
        }
        matched
    }

    pub fn activity(&self, request: ProblemActivityRequest) -> Vec<ProblemActivityEntry> {
        let store = self.store.read().expect("problem bank read lock");
        let mut entries = store
            .by_id
            .values()
            .filter(|item| request.matches_problem(item))
            .flat_map(|item| {
                item.usage_history.iter().filter_map(|history| {
                    if request.matches_history(history) {
                        Some(ProblemActivityEntry {
                            problem_id: item.id.clone(),
                            title: item.title.clone(),
                            mode: item.mode.clone(),
                            level_band: item.level_band.clone(),
                            topic: item.topic.clone(),
                            target_context: item.target_context.clone(),
                            source: item.source.clone(),
                            pinned: item.pinned,
                            successful: history.successful,
                            occurred_at_unix: history.occurred_at_unix,
                            note: history.note.clone(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| {
            b.occurred_at_unix
                .cmp(&a.occurred_at_unix)
                .then_with(|| a.problem_id.cmp(&b.problem_id))
        });
        if entries.len() > request.limit {
            entries.truncate(request.limit);
        }
        entries
    }

    pub fn history(&self, id: &str) -> Option<Vec<ProblemUsageHistory>> {
        let store = self.store.read().expect("problem bank read lock");
        store
            .by_id
            .get(id)
            .map(|item| item.usage_history.clone())
    }

    pub fn stats(&self) -> ProblemBankStats {
        let store = self.store.read().expect("problem bank read lock");
        let mut by_mode = HashMap::new();
        let mut by_level_band = HashMap::new();
        let mut by_context = HashMap::new();
        for item in store.all_items() {
            *by_mode.entry(item.mode.clone()).or_insert(0) += 1;
            *by_level_band.entry(item.level_band.clone()).or_insert(0) += 1;
            *by_context.entry(item.target_context.clone()).or_insert(0) += 1;
        }

        ProblemBankStats {
            total: store.total_count(),
            seeded: store.seeded.len(),
            custom: store.custom.len(),
            pinned: store.all_items().iter().filter(|item| item.pinned).count(),
            total_usage: store.all_items().iter().map(|item| item.usage_count as usize).sum(),
            by_mode,
            by_level_band,
            by_context,
            by_source: group_by_source(&store.all_items()),
        }
    }

    pub fn insights(&self, request: ProblemActivityRequest) -> ProblemBankInsights {
        let store = self.store.read().expect("problem bank read lock");
        let matched_problems = store
            .by_id
            .values()
            .filter(|item| request.matches_problem(item))
            .cloned()
            .collect::<Vec<_>>();

        let mut total_history_entries = 0usize;
        let mut successful_history_entries = 0usize;
        let mut by_mode_activity = HashMap::new();
        let mut by_context_activity = HashMap::new();
        let mut by_source_activity = HashMap::new();
        let mut top_used_problems = Vec::new();

        for item in matched_problems {
            let matched_history = item
                .usage_history
                .iter()
                .filter(|history| request.matches_history(history))
                .cloned()
                .collect::<Vec<_>>();

            if matched_history.is_empty() {
                continue;
            }

            let successful = matched_history.iter().filter(|entry| entry.successful).count();
            total_history_entries += matched_history.len();
            successful_history_entries += successful;
            *by_mode_activity.entry(item.mode.clone()).or_insert(0) += matched_history.len();
            *by_context_activity
                .entry(item.target_context.clone())
                .or_insert(0) += matched_history.len();
            *by_source_activity.entry(item.source.clone()).or_insert(0) += matched_history.len();

            let last_used_unix = matched_history
                .iter()
                .map(|entry| entry.occurred_at_unix)
                .max()
                .unwrap_or(0);
            top_used_problems.push(ProblemUsageSummary {
                problem_id: item.id,
                title: item.title,
                mode: item.mode,
                target_context: item.target_context,
                source: item.source,
                usage_count: matched_history.len(),
                success_count: successful,
                last_used_unix,
                pinned: item.pinned,
            });
        }

        top_used_problems.sort_by(|a, b| {
            b.usage_count
                .cmp(&a.usage_count)
                .then_with(|| b.success_count.cmp(&a.success_count))
                .then_with(|| b.last_used_unix.cmp(&a.last_used_unix))
                .then_with(|| a.problem_id.cmp(&b.problem_id))
        });
        if top_used_problems.len() > request.limit {
            top_used_problems.truncate(request.limit);
        }

        ProblemBankInsights {
            total_history_entries,
            successful_history_entries,
            failed_history_entries: total_history_entries.saturating_sub(successful_history_entries),
            overall_success_rate: if total_history_entries == 0 {
                0.0
            } else {
                successful_history_entries as f64 / total_history_entries as f64
            },
            by_mode_activity,
            by_context_activity,
            by_source_activity,
            top_used_problems,
        }
    }

    pub fn recommend(&self, request: ProblemRecommendationRequest) -> Vec<ProblemRecord> {
        let store = self.store.read().expect("problem bank read lock");
        let mut ranked = store
            .all_items()
            .into_iter()
            .map(|item| {
                let score = recommendation_score(&item, &request);
                (score, item)
            })
            .filter(|(score, _)| *score > 0)
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.sort_order.cmp(&b.1.sort_order))
                .then_with(|| a.1.id.cmp(&b.1.id))
        });

        ranked
            .into_iter()
            .take(request.limit.max(1).min(20))
            .map(|(_, item)| item)
            .collect()
    }

    pub fn review_queue(&self, request: ProblemRecommendationRequest) -> Vec<ProblemRecord> {
        let store = self.store.read().expect("problem bank read lock");
        let mut ranked = store
            .all_items()
            .into_iter()
            .map(|item| {
                let score = review_queue_score(&item, &request);
                (score, item)
            })
            .filter(|(score, _)| *score > 0)
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.last_used_unix.cmp(&a.1.last_used_unix))
                .then_with(|| a.1.sort_order.cmp(&b.1.sort_order))
                .then_with(|| a.1.id.cmp(&b.1.id))
        });

        ranked
            .into_iter()
            .take(request.limit.max(1).min(20))
            .map(|(_, item)| item)
            .collect()
    }

    pub fn weakness_queue(&self, request: ProblemRecommendationRequest) -> ProblemWeaknessQueue {
        let modes = ["reading", "listening", "speaking", "rescue"];
        let groups = modes
            .into_iter()
            .map(|mode| {
                let mut mode_request = request.clone();
                mode_request.preferred_mode = Some(mode.to_string());
                let items = self.review_queue(mode_request);
                ProblemWeaknessGroup {
                    mode: mode.to_string(),
                    total_candidates: items.len(),
                    items,
                }
            })
            .filter(|group| !group.items.is_empty())
            .collect::<Vec<_>>();

        ProblemWeaknessQueue { groups }
    }

    pub fn dashboard(
        &self,
        recommendation: ProblemRecommendationRequest,
        activity: ProblemActivityRequest,
    ) -> ProblemBankDashboard {
        let review_queue = self.review_queue(recommendation.clone());
        let weakness_queue = self.weakness_queue(recommendation);
        let recommended_next_mode = weakness_queue
            .groups
            .first()
            .map(|group| group.mode.clone());
        let stale_problems = self.stale_problems(ProblemStaleRequest::default());

        ProblemBankDashboard {
            stats: self.stats(),
            insights: self.insights(activity),
            review_queue,
            weakness_queue,
            recommended_next_mode,
            stale_problems,
        }
    }

    pub fn stale_problems(&self, request: ProblemStaleRequest) -> Vec<ProblemStaleEntry> {
        let store = self.store.read().expect("problem bank read lock");
        let now = current_unix_seconds();
        let stale_after_seconds = request
            .stale_after_days
            .max(1)
            .saturating_mul(24 * 60 * 60);

        let mut items = store
            .by_id
            .values()
            .filter(|item| request.matches(item))
            .filter_map(|item| {
                let idle_seconds = if item.last_used_unix == 0 {
                    now
                } else {
                    now.saturating_sub(item.last_used_unix)
                };
                if idle_seconds < stale_after_seconds {
                    return None;
                }

                Some(ProblemStaleEntry {
                    problem_id: item.id.clone(),
                    title: item.title.clone(),
                    mode: item.mode.clone(),
                    target_context: item.target_context.clone(),
                    source: item.source.clone(),
                    pinned: item.pinned,
                    last_used_unix: item.last_used_unix,
                    idle_days: idle_seconds / (24 * 60 * 60),
                    usage_count: item.usage_count,
                })
            })
            .collect::<Vec<_>>();

        items.sort_by(|a, b| {
            b.idle_days
                .cmp(&a.idle_days)
                .then_with(|| a.problem_id.cmp(&b.problem_id))
        });
        if items.len() > request.limit {
            items.truncate(request.limit);
        }
        items
    }

    pub fn save_generated_set(
        &self,
        generated: &GeneratedProblemSet,
        source: ProblemSaveSource,
    ) -> Result<SavedProblemSet, ProblemBankSaveError> {
        let mut custom_items = generated.items.clone();
        for (index, item) in custom_items.iter_mut().enumerate() {
            item.id = saved_problem_id(&item.id, &generated.summary, index);
            item.tags.push("saved".to_string());
            item.tags.push(source.as_tag().to_string());
            item.sort_order = 1000 + index as u32;
            item.source = source.as_tag().to_string();
        }

        let mut store = self.store.write().expect("problem bank write lock");
        for item in custom_items.iter().cloned() {
            store.upsert_custom(item);
        }
        let stats = ProblemBankStats {
            total: store.total_count(),
            seeded: store.seeded.len(),
            custom: store.custom.len(),
            pinned: store.all_items().iter().filter(|item| item.pinned).count(),
            total_usage: store.all_items().iter().map(|item| item.usage_count as usize).sum(),
            by_mode: HashMap::new(),
            by_level_band: HashMap::new(),
            by_context: HashMap::new(),
            by_source: HashMap::new(),
        };
        let saved_items = custom_items.clone();
        if let Some(path) = self.persisted_path.as_ref() {
            persist_custom_records(path, &store.custom.values().cloned().collect::<Vec<_>>())?;
        }

        Ok(SavedProblemSet {
            source: source.as_tag().to_string(),
            saved_count: saved_items.len(),
            items: saved_items,
            total_custom: stats.custom,
            total_all: stats.total,
        })
    }

    pub fn clone_problem(
        &self,
        id: &str,
        source: ProblemSaveSource,
    ) -> Result<SavedProblemSet, ProblemBankSaveError> {
        let original = {
            let store = self.store.read().expect("problem bank read lock");
            store
                .by_id
                .get(id)
                .cloned()
                .ok_or(ProblemBankSaveError::ProblemNotFound)?
        };

        let generated = GeneratedProblemSet {
            source_text: original.prompt.clone(),
            summary: original.title.clone(),
            target_context: original.target_context.clone(),
            level_band: original.level_band.clone(),
            topic: original.topic.clone(),
            items: vec![original],
        };

        self.save_generated_set(&generated, source)
    }

    pub fn delete_custom(&self, id: &str) -> Result<DeletedProblemRecord, ProblemBankDeleteError> {
        let mut store = self.store.write().expect("problem bank write lock");
        let removed = store
            .custom
            .remove(id)
            .ok_or(ProblemBankDeleteError::NotFound)?;
        store.rebuild_index();

        if let Some(path) = self.persisted_path.as_ref() {
            persist_custom_records(path, &store.custom.values().cloned().collect::<Vec<_>>())
                .map_err(ProblemBankDeleteError::Persist)?;
        }

        Ok(DeletedProblemRecord {
            id: removed.id,
            remaining_custom: store.custom.len(),
            remaining_total: store.total_count(),
        })
    }

    pub fn update_custom(
        &self,
        id: &str,
        update: ProblemRecordUpdate,
    ) -> Result<ProblemRecord, ProblemBankUpdateError> {
        let mut store = self.store.write().expect("problem bank write lock");
        let item = store
            .custom
            .get_mut(id)
            .ok_or(ProblemBankUpdateError::NotFound)?;

        if let Some(title) = update.title {
            item.title = title;
        }
        if let Some(prompt) = update.prompt {
            item.prompt = prompt;
        }
        if let Some(wm_support) = update.wm_support {
            item.wm_support = wm_support;
        }
        if let Some(success_check) = update.success_check {
            item.success_check = success_check;
        }
        if let Some(tags) = update.tags {
            item.tags = tags;
        }
        if let Some(notes) = update.notes {
            item.notes = notes;
        }
        if let Some(pinned) = update.pinned {
            item.pinned = pinned;
        }

        let updated = item.clone();
        store.rebuild_index();
        if let Some(path) = self.persisted_path.as_ref() {
            persist_custom_records(path, &store.custom.values().cloned().collect::<Vec<_>>())
                .map_err(ProblemBankUpdateError::Persist)?;
        }
        Ok(updated)
    }

    pub fn record_usage(
        &self,
        id: &str,
        event: ProblemUsageEvent,
    ) -> Result<ProblemRecord, ProblemBankUpdateError> {
        let mut store = self.store.write().expect("problem bank write lock");
        let item = store
            .custom
            .get_mut(id)
            .ok_or(ProblemBankUpdateError::NotFound)?;

        item.usage_count = item.usage_count.saturating_add(1);
        if event.successful {
            item.success_count = item.success_count.saturating_add(1);
        }
        item.last_used_unix = event.occurred_at_unix.unwrap_or_else(current_unix_seconds);
        item.usage_history.push(ProblemUsageHistory {
            successful: event.successful,
            occurred_at_unix: item.last_used_unix,
            note: event.append_note.clone().unwrap_or_default(),
        });
        if let Some(note) = event.append_note {
            if item.notes.is_empty() {
                item.notes = note;
            } else {
                item.notes = format!("{}\n{}", item.notes, note);
            }
        }

        let updated = item.clone();
        store.rebuild_index();
        if let Some(path) = self.persisted_path.as_ref() {
            persist_custom_records(path, &store.custom.values().cloned().collect::<Vec<_>>())
                .map_err(ProblemBankUpdateError::Persist)?;
        }
        Ok(updated)
    }

    pub fn generate(&self, request: ProblemGenerationRequest) -> GeneratedProblemSet {
        let normalized = normalize_text(&request.text);
        let sentences = split_sentences(&normalized);
        let summary = summarize(&sentences, &normalized);
        let focus_text = sentences
            .first()
            .cloned()
            .unwrap_or_else(|| normalized.clone());
        let support_text = support_focus(&normalized).unwrap_or_else(|| focus_text.clone());
        let level_band = request
            .level_band
            .unwrap_or_else(|| "toeic_600_700".to_string());
        let target_context = request
            .target_context
            .unwrap_or_else(|| "general".to_string());
        let topic = request.topic.unwrap_or_else(|| infer_topic(&target_context, &normalized));
        let base_id = generated_id(&normalized, &target_context, &level_band);
        let profile = generated_profile(&target_context, &topic, &focus_text, &support_text, &summary);

        let items = vec![
            generated_problem(
                format!("{base_id}_read"),
                profile.reading_title,
                "reading",
                &level_band,
                &topic,
                &target_context,
                profile.reading_prompt,
                profile.reading_support,
                profile.reading_success,
                10,
                &["generated", "core_lock"],
            ),
            generated_problem(
                format!("{base_id}_listen"),
                profile.listening_title,
                "listening",
                &level_band,
                &topic,
                &target_context,
                profile.listening_prompt,
                profile.listening_support,
                profile.listening_success,
                20,
                &["generated", "pause_recall"],
            ),
            generated_problem(
                format!("{base_id}_speak"),
                profile.speaking_title,
                "speaking",
                &level_band,
                &topic,
                &target_context,
                profile.speaking_prompt,
                profile.speaking_support,
                profile.speaking_success,
                30,
                &["generated", "short_unit"],
            ),
            generated_problem(
                format!("{base_id}_rescue"),
                profile.rescue_title,
                "rescue",
                &level_band,
                "rescue",
                &target_context,
                profile.rescue_prompt,
                profile.rescue_support,
                profile.rescue_success,
                40,
                &["generated", "rescue"],
            ),
        ];

        GeneratedProblemSet {
            source_text: normalized,
            summary,
            target_context,
            level_band,
            topic,
            items,
        }
    }
}

#[derive(Clone)]
struct ProblemStore {
    seeded: Vec<ProblemRecord>,
    custom: HashMap<String, ProblemRecord>,
    by_id: HashMap<String, ProblemRecord>,
}

impl ProblemStore {
    fn new(seeded: Vec<ProblemRecord>, custom: Vec<ProblemRecord>) -> Self {
        let mut store = Self {
            seeded,
            custom: HashMap::new(),
            by_id: HashMap::new(),
        };
        for item in custom {
            store.custom.insert(item.id.clone(), item);
        }
        store.rebuild_index();
        store
    }

    fn rebuild_index(&mut self) {
        let mut index = self
            .seeded
            .iter()
            .cloned()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();
        index.extend(self.custom.iter().map(|(id, item)| (id.clone(), item.clone())));
        self.by_id = index;
    }

    fn upsert_custom(&mut self, item: ProblemRecord) {
        self.custom.insert(item.id.clone(), item);
        self.rebuild_index();
    }

    fn all_items(&self) -> Vec<ProblemRecord> {
        let mut items = self.seeded.clone();
        items.extend(self.custom.values().cloned());
        items
    }

    fn total_count(&self) -> usize {
        self.seeded.len() + self.custom.len()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProblemRecord {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub level_band: String,
    pub topic: String,
    pub target_context: String,
    pub prompt: String,
    pub wm_support: String,
    pub success_check: String,
    pub tags: Vec<String>,
    pub sort_order: u32,
    #[serde(default = "default_problem_source")]
    pub source: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub usage_count: u32,
    #[serde(default)]
    pub success_count: u32,
    #[serde(default)]
    pub last_used_unix: u64,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub usage_history: Vec<ProblemUsageHistory>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemActivityEntry {
    pub problem_id: String,
    pub title: String,
    pub mode: String,
    pub level_band: String,
    pub topic: String,
    pub target_context: String,
    pub source: String,
    pub pinned: bool,
    pub successful: bool,
    pub occurred_at_unix: u64,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemGenerationRequest {
    pub text: String,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedProblemSet {
    pub source_text: String,
    pub summary: String,
    pub target_context: String,
    pub level_band: String,
    pub topic: String,
    pub items: Vec<ProblemRecord>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemBankStats {
    pub total: usize,
    pub seeded: usize,
    pub custom: usize,
    pub pinned: usize,
    pub total_usage: usize,
    pub by_mode: HashMap<String, usize>,
    pub by_level_band: HashMap<String, usize>,
    pub by_context: HashMap<String, usize>,
    pub by_source: HashMap<String, usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemBankInsights {
    pub total_history_entries: usize,
    pub successful_history_entries: usize,
    pub failed_history_entries: usize,
    pub overall_success_rate: f64,
    pub by_mode_activity: HashMap<String, usize>,
    pub by_context_activity: HashMap<String, usize>,
    pub by_source_activity: HashMap<String, usize>,
    pub top_used_problems: Vec<ProblemUsageSummary>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemUsageSummary {
    pub problem_id: String,
    pub title: String,
    pub mode: String,
    pub target_context: String,
    pub source: String,
    pub usage_count: usize,
    pub success_count: usize,
    pub last_used_unix: u64,
    pub pinned: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemWeaknessQueue {
    pub groups: Vec<ProblemWeaknessGroup>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemWeaknessGroup {
    pub mode: String,
    pub total_candidates: usize,
    pub items: Vec<ProblemRecord>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemBankDashboard {
    pub stats: ProblemBankStats,
    pub insights: ProblemBankInsights,
    pub review_queue: Vec<ProblemRecord>,
    pub weakness_queue: ProblemWeaknessQueue,
    pub recommended_next_mode: Option<String>,
    pub stale_problems: Vec<ProblemStaleEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProblemStaleEntry {
    pub problem_id: String,
    pub title: String,
    pub mode: String,
    pub target_context: String,
    pub source: String,
    pub pinned: bool,
    pub last_used_unix: u64,
    pub idle_days: u64,
    pub usage_count: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemRecordUpdate {
    pub title: Option<String>,
    pub prompt: Option<String>,
    pub wm_support: Option<String>,
    pub success_check: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemUsageEvent {
    pub successful: bool,
    pub occurred_at_unix: Option<u64>,
    pub append_note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProblemUsageHistory {
    pub successful: bool,
    pub occurred_at_unix: u64,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProblemRecommendationRequest {
    pub preferred_mode: Option<String>,
    pub target_context: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub focus_tag: Option<String>,
    pub prefer_review: bool,
    pub avoid_mastered: bool,
    pub limit: usize,
}

#[derive(Clone, Debug)]
pub struct ProblemActivityRequest {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub query: Option<String>,
    pub successful: Option<bool>,
    pub pinned_only: bool,
    pub limit: usize,
}

impl Default for ProblemActivityRequest {
    fn default() -> Self {
        Self {
            mode: None,
            level_band: None,
            topic: None,
            target_context: None,
            source: None,
            query: None,
            successful: None,
            pinned_only: false,
            limit: 20,
        }
    }
}

impl ProblemActivityRequest {
    fn matches_problem(&self, item: &ProblemRecord) -> bool {
        if self.pinned_only && !item.pinned {
            return false;
        }
        if let Some(mode) = self.mode.as_deref() {
            if !item.mode.eq_ignore_ascii_case(mode) {
                return false;
            }
        }
        if let Some(level_band) = self.level_band.as_deref() {
            if !item.level_band.eq_ignore_ascii_case(level_band) {
                return false;
            }
        }
        if let Some(topic) = self.topic.as_deref() {
            if !item.topic.eq_ignore_ascii_case(topic) {
                return false;
            }
        }
        if let Some(target_context) = self.target_context.as_deref() {
            if !item.target_context.eq_ignore_ascii_case(target_context) {
                return false;
            }
        }
        if let Some(source) = self.source.as_deref() {
            if !item.source.eq_ignore_ascii_case(source) {
                return false;
            }
        }
        if let Some(query) = self.query.as_deref() {
            let lowered = query.to_ascii_lowercase();
            let haystacks = [
                item.title.as_str(),
                item.prompt.as_str(),
                item.notes.as_str(),
                item.topic.as_str(),
                item.target_context.as_str(),
            ];
            let tag_match = item
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(&lowered));
            let text_match = haystacks
                .iter()
                .any(|value| value.to_ascii_lowercase().contains(&lowered));
            if !tag_match && !text_match {
                return false;
            }
        }
        true
    }

    fn matches_history(&self, history: &ProblemUsageHistory) -> bool {
        match self.successful {
            Some(successful) => history.successful == successful,
            None => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProblemStaleRequest {
    pub mode: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub pinned_only: bool,
    pub stale_after_days: u64,
    pub limit: usize,
}

impl Default for ProblemStaleRequest {
    fn default() -> Self {
        Self {
            mode: None,
            target_context: None,
            source: None,
            pinned_only: false,
            stale_after_days: 7,
            limit: 10,
        }
    }
}

impl ProblemStaleRequest {
    fn matches(&self, item: &ProblemRecord) -> bool {
        if self.pinned_only && !item.pinned {
            return false;
        }
        if let Some(mode) = self.mode.as_deref() {
            if !item.mode.eq_ignore_ascii_case(mode) {
                return false;
            }
        }
        if let Some(target_context) = self.target_context.as_deref() {
            if !item.target_context.eq_ignore_ascii_case(target_context) {
                return false;
            }
        }
        if let Some(source) = self.source.as_deref() {
            if !item.source.eq_ignore_ascii_case(source) {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemSaveSource {
    Generated,
    Reviewed,
}

impl ProblemSaveSource {
    fn as_tag(&self) -> &'static str {
        match self {
            Self::Generated => "generated",
            Self::Reviewed => "reviewed",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SavedProblemSet {
    pub source: String,
    pub saved_count: usize,
    pub items: Vec<ProblemRecord>,
    pub total_custom: usize,
    pub total_all: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeletedProblemRecord {
    pub id: String,
    pub remaining_custom: usize,
    pub remaining_total: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ProblemBankSaveError {
    #[error("problem not found")]
    ProblemNotFound,
    #[error("failed to create problem bank directory")]
    CreateDirectory(#[source] std::io::Error),
    #[error("failed to serialize saved problems")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to write saved problems")]
    Write(#[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ProblemBankDeleteError {
    #[error("problem not found")]
    NotFound,
    #[error("failed to persist updated problem bank")]
    Persist(#[source] ProblemBankSaveError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProblemBankUpdateError {
    #[error("problem not found")]
    NotFound,
    #[error("failed to persist updated problem bank")]
    Persist(#[source] ProblemBankSaveError),
}

#[derive(Clone, Debug)]
pub struct ProblemFilter {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
    pub source: Option<String>,
    pub tag: Option<String>,
    pub pinned_only: bool,
    pub query: Option<String>,
    pub limit: usize,
}

impl Default for ProblemFilter {
    fn default() -> Self {
        Self {
            mode: None,
            level_band: None,
            topic: None,
            target_context: None,
            source: None,
            tag: None,
            pinned_only: false,
            query: None,
            limit: 20,
        }
    }
}

impl ProblemFilter {
    fn matches(&self, item: &ProblemRecord) -> bool {
        if let Some(mode) = self.mode.as_deref() {
            if !item.mode.eq_ignore_ascii_case(mode) {
                return false;
            }
        }
        if let Some(level_band) = self.level_band.as_deref() {
            if !item.level_band.eq_ignore_ascii_case(level_band) {
                return false;
            }
        }
        if let Some(topic) = self.topic.as_deref() {
            if !item.topic.eq_ignore_ascii_case(topic) {
                return false;
            }
        }
        if let Some(target_context) = self.target_context.as_deref() {
            if !item.target_context.eq_ignore_ascii_case(target_context) {
                return false;
            }
        }
        if let Some(source) = self.source.as_deref() {
            if !item.source.eq_ignore_ascii_case(source) {
                return false;
            }
        }
        if let Some(tag) = self.tag.as_deref() {
            if !item
                .tags
                .iter()
                .any(|item_tag| item_tag.eq_ignore_ascii_case(tag))
            {
                return false;
            }
        }
        if self.pinned_only && !item.pinned {
            return false;
        }
        if let Some(query) = self.query.as_deref() {
            let lowered = query.to_ascii_lowercase();
            let haystacks = [
                item.title.as_str(),
                item.prompt.as_str(),
                item.wm_support.as_str(),
                item.success_check.as_str(),
                item.topic.as_str(),
                item.target_context.as_str(),
            ];
            let tag_match = item
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(&lowered));
            let text_match = haystacks
                .iter()
                .any(|value| value.to_ascii_lowercase().contains(&lowered));
            if !tag_match && !text_match {
                return false;
            }
        }
        true
    }
}

fn load_custom_records(path: &Path) -> Result<Vec<ProblemRecord>, std::io::Error> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    let records = serde_json::from_str::<Vec<ProblemRecord>>(&raw).unwrap_or_default();
    Ok(records)
}

fn recommendation_score(item: &ProblemRecord, request: &ProblemRecommendationRequest) -> i32 {
    let mut score = 1;

    if let Some(mode) = request.preferred_mode.as_deref() {
        if item.mode.eq_ignore_ascii_case(mode) {
            score += 5;
        }
    }
    if let Some(target_context) = request.target_context.as_deref() {
        if item.target_context.eq_ignore_ascii_case(target_context) {
            score += 4;
        }
    }
    if let Some(level_band) = request.level_band.as_deref() {
        if item.level_band.eq_ignore_ascii_case(level_band) {
            score += 3;
        }
    }
    if let Some(topic) = request.topic.as_deref() {
        if item.topic.eq_ignore_ascii_case(topic) {
            score += 2;
        }
    }
    if let Some(focus_tag) = request.focus_tag.as_deref() {
        if item
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(focus_tag) || tag.to_ascii_lowercase().contains(&focus_tag.to_ascii_lowercase()))
        {
            score += 4;
        }
    }

    if item.tags.iter().any(|tag| tag == "saved") {
        score += 1;
    }
    if item.pinned {
        score += 2;
    }
    if request.prefer_review {
        score += review_signal(item);
    } else {
        score += (item.success_count.min(3)) as i32;
    }
    if request.avoid_mastered && is_mastered(item) {
        score -= 6;
    }

    score
}

fn review_queue_score(item: &ProblemRecord, request: &ProblemRecommendationRequest) -> i32 {
    let mut score = recommendation_score(item, request);
    score += review_signal(item);
    if is_mastered(item) {
        score -= if request.avoid_mastered { 10 } else { 4 };
    }
    if item.usage_count == 0 {
        score += 2;
    }
    score
}

fn review_signal(item: &ProblemRecord) -> i32 {
    let failures = item.usage_count.saturating_sub(item.success_count);
    let mut score = 0i32;
    score += (failures.min(3) * 2) as i32;
    if last_usage_was_failure(item) {
        score += 4;
    }
    if item.usage_count > 0 && item.success_count == 0 {
        score += 2;
    }
    score
}

fn is_mastered(item: &ProblemRecord) -> bool {
    item.usage_count >= 2
        && item.success_count >= 2
        && item.success_count.saturating_mul(10) >= item.usage_count.saturating_mul(8)
}

fn last_usage_was_failure(item: &ProblemRecord) -> bool {
    item.usage_history
        .iter()
        .max_by_key(|entry| entry.occurred_at_unix)
        .map(|entry| !entry.successful)
        .unwrap_or(false)
}

fn group_by_source(items: &[ProblemRecord]) -> HashMap<String, usize> {
    let mut by_source = HashMap::new();
    for item in items {
        *by_source.entry(item.source.clone()).or_insert(0) += 1;
    }
    by_source
}

fn persist_custom_records(path: &Path, records: &[ProblemRecord]) -> Result<(), ProblemBankSaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(ProblemBankSaveError::CreateDirectory)?;
    }
    let payload = serde_json::to_string_pretty(records)?;
    fs::write(path, payload).map_err(ProblemBankSaveError::Write)
}

fn saved_problem_id(base_id: &str, summary: &str, index: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(base_id.as_bytes());
    hasher.update(summary.as_bytes());
    hasher.update(index.to_string().as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!("saved_{}", &digest[..12])
}

fn seeded_records() -> Vec<ProblemRecord> {
    vec![
        problem("pb_read_001", "Core Lock: Meeting Goal", "reading", "toeic_600_700", "meeting", "meeting", "Read only the meeting goal first: 'Today's goal is to confirm the release date.' Stop before the support details.", "Keep the core sentence visible before adding schedule detail.", "You can restate the goal in one short sentence.", 10, &["core_lock", "goal_first"]),
        problem("pb_read_002", "Support Attach: Research Limitation", "reading", "toeic_750_800", "research", "research", "Read the limitation sentence, then add one support phrase about future work without rereading everything.", "Attach only one support detail after the main limitation is stable.", "You can explain the limitation and one next step.", 20, &["support_attach", "research"]),
        problem("pb_read_003", "Core Lock: Shipment Delay", "reading", "toeic_750_800", "meeting", "meeting", "Read the delay notice and keep only the cause and new arrival day in mind.", "Ignore optional wording until cause and consequence are fixed.", "You can say what changed and why.", 30, &["business_notice", "core_lock"]),
        problem("pb_listen_001", "Pause Recall: Boarding Notice", "listening", "starter", "daily", "daily", "Listen to one boarding notice chunk, pause, and say only the action you need to take.", "The pause protects one action before the next audio arrives.", "You can state the action before replaying.", 40, &["pause_recall", "daily"]),
        problem("pb_listen_002", "Meaning Hold: Invoice Rule", "listening", "toeic_750_800", "meeting", "meeting", "After hearing the invoice rule, restate only the condition that changes the process.", "This rewards gist retention instead of word-by-word memory.", "You can explain when the invoice must be forwarded.", 50, &["meaning_hold", "business_rule"]),
        problem("pb_listen_003", "Pause Recall: Research Result", "listening", "toeic_600_700", "research", "research", "Pause after the result sentence and keep only the two outcome directions: overload and accuracy.", "Short pauses reduce the need to carry both outcomes through the full sentence.", "You can say what went down and what went up.", 60, &["result_hold", "research"]),
        problem("pb_speak_001", "Opener Only: Self Introduction", "speaking", "starter", "self_intro", "self_intro", "Say only the opener: 'Hello, I study learning support.' Stop there and do not continue yet.", "A short opener reduces pressure to plan the whole answer at once.", "You can start clearly without freezing.", 70, &["opener_only", "self_intro"]),
        problem("pb_speak_002", "Two-Step Link: Status Update", "speaking", "toeic_750_800", "meeting", "meeting", "Say two short steps: 'The client approved the design. The schedule is still open.'", "Two short units are safer than one long report sentence.", "You can deliver the update without collapsing the second idea.", 80, &["two_step_link", "status_update"]),
        problem("pb_speak_003", "Short Unit: Research Claim", "speaking", "toeic_600_700", "research", "research", "Deliver the claim in three short parts: 'Lower load helps comprehension. Our interface supports that. The effect is practical.'", "Short units reduce holding demand while preserving meaning.", "You can finish the claim without restarting.", 90, &["short_unit", "research_claim"]),
        problem("pb_rescue_001", "Rescue: Ask for the Deadline First", "rescue", "toeic_750_800", "rescue", "meeting", "Practice saying: 'Could you tell me the deadline first, and then explain the rest?'", "This creates a single anchor before details pile up.", "You can use the phrase quickly under pressure.", 100, &["deadline_first", "rescue"]),
        problem("pb_rescue_002", "Rescue: Ask for One Condition", "rescue", "toeic_750_800", "rescue", "meeting", "Practice saying: 'Could we go through one condition at a time?'", "It splits complex business instructions into smaller chunks.", "You can request one condition without apology spirals.", 110, &["one_condition", "rescue"]),
        problem("pb_rescue_003", "Rescue: Ask for Main Point", "rescue", "starter", "rescue", "daily", "Practice saying: 'Can you tell me the main point first?'", "Main-point-first phrasing lowers memory load immediately.", "You can ask for the core idea fast.", 120, &["main_point", "rescue"]),
        problem("pb_read_004", "Core Lock: Store Policy", "reading", "toeic_750_800", "daily", "daily", "Read only the new store rule first: returns now require either a receipt or a digital purchase record.", "Lock the rule before you try to remember exceptions.", "You can state the new requirement clearly.", 130, &["core_lock", "policy"]),
        problem("pb_read_005", "Support Attach: Seminar Overview", "reading", "toeic_750_800", "research", "research", "Read the seminar topic first, then attach the time-pressure detail only after the topic is stable.", "Delay the extra condition until the main theme is stable.", "You can explain the seminar topic and one key detail.", 140, &["support_attach", "seminar"]),
        problem("pb_read_006", "Core Lock: Budget Shift", "reading", "toeic_600_700", "meeting", "meeting", "Read the budget decision and keep only the postponed work and the new priority.", "Reduce the sentence to two business actions before adding context.", "You can say what was postponed and what was prioritized.", 150, &["budget", "core_lock"]),
        problem("pb_read_007", "Meaning Route: Customer Survey", "reading", "toeic_750_800", "research", "research", "Read the survey summary and hold only the strongest user preference before you look at the lower-ranked item.", "This protects the ranking relation before the contrast arrives.", "You can explain what users valued most.", 160, &["ranking", "survey"]),
        problem("pb_read_008", "Core Lock: Maintenance Notice", "reading", "toeic_600_700", "daily", "daily", "Read the maintenance notice and stop after the practical implication for staff.", "Keep the action consequence before the schedule detail overloads you.", "You can say what staff should do differently.", 170, &["notice", "action_first"]),
        problem("pb_listen_004", "Pause Recall: Venue Change", "listening", "toeic_750_800", "meeting", "meeting", "Pause after the venue change announcement and say the new room before you continue.", "Hold the new location first; the reason can come second.", "You can state where the briefing will happen.", 180, &["pause_recall", "venue_change"]),
        problem("pb_listen_005", "Meaning Hold: Subscription Renewal", "listening", "toeic_750_800", "daily", "daily", "After hearing the renewal notice, keep only the renewal day and the condition for stopping it.", "This reduces memory demand to one date and one condition.", "You can explain when renewal happens and how to prevent it.", 190, &["renewal", "condition_hold"]),
        problem("pb_listen_006", "Pause Recall: Delivery Window", "listening", "toeic_600_700", "daily", "daily", "Pause after the delivery window sentence and say the time range only.", "Holding the time range first makes the follow-up action easier.", "You can repeat the delivery window accurately.", 200, &["time_window", "pause_recall"]),
        problem("pb_listen_007", "Meaning Hold: Interview Plan", "listening", "toeic_750_800", "meeting", "meeting", "Listen to the interview plan and keep only the follow-up action after each session.", "Ignore the number of candidates until the action is stable.", "You can explain what must be completed after each interview.", 210, &["interview", "meaning_hold"]),
        problem("pb_listen_008", "Pause Recall: Research Procedure", "listening", "toeic_600_700", "research", "research", "Pause after the procedure sentence and say the order of the first two actions.", "Short pauses help preserve order without replaying the whole line.", "You can restate the first two procedure steps.", 220, &["procedure", "ordered_steps"]),
        problem("pb_speak_004", "Opener Only: Business Update", "speaking", "toeic_600_700", "meeting", "meeting", "Start with only: 'Here is the short update.' Stop there before adding detail.", "An opener buys structure before the business content begins.", "You can begin without planning the full report.", 230, &["opener_only", "business_update"]),
        problem("pb_speak_005", "Short Unit: Delay Explanation", "speaking", "toeic_750_800", "meeting", "meeting", "Say the delay in three short units: 'The supplier changed the date. Inventory is not final. We need one more day.'", "Three short units are safer than one dense explanation.", "You can explain the delay without losing the last step.", 240, &["short_unit", "delay"]),
        problem("pb_speak_006", "Two-Step Link: Store Policy", "speaking", "toeic_750_800", "daily", "daily", "Say the policy in two short linked steps: 'Returns need proof of purchase. A digital record is also fine.'", "Link only two ideas at a time to avoid collapse.", "You can explain the policy in two stable pieces.", 250, &["two_step_link", "policy"]),
        problem("pb_speak_007", "Short Unit: Research Limitation", "speaking", "toeic_750_800", "research", "research", "Deliver the limitation in short parts: 'We tested short texts first. Live conversation data is still limited. That is our next target.'", "Short units keep the limitation and next step from blending together.", "You can say the limitation and next step cleanly.", 260, &["short_unit", "limitation"]),
        problem("pb_speak_008", "Two-Step Link: Priority Choice", "speaking", "toeic_600_700", "meeting", "meeting", "Link two short choices: 'I can finish the report first. Or I can answer the client email first.'", "Separating the options reduces planning load.", "You can present both options without merging them.", 270, &["priority", "two_step_link"]),
        problem("pb_rescue_004", "Rescue: Ask for the Rule First", "rescue", "toeic_750_800", "rescue", "daily", "Practice saying: 'Could you tell me the rule first, and then the example?'", "This anchors one policy statement before examples compete for attention.", "You can ask for the rule without hesitation.", 280, &["rule_first", "rescue"]),
        problem("pb_rescue_005", "Rescue: Ask Which Task Comes First", "rescue", "toeic_600_700", "rescue", "meeting", "Practice saying: 'Which task comes first for me?'", "One first-task question reduces overload when multiple tasks appear together.", "You can identify the first task quickly.", 290, &["first_task", "rescue"]),
        problem("pb_rescue_006", "Rescue: Ask for the New Room Only", "rescue", "toeic_750_800", "rescue", "meeting", "Practice saying: 'Could you repeat only the new room?'", "This keeps the location separate from the reason for the change.", "You can recover the one missing detail you need.", 300, &["room_repeat", "rescue"]),
        problem("pb_rescue_007", "Rescue: Ask for One Date at a Time", "rescue", "toeic_750_800", "rescue", "daily", "Practice saying: 'Could we go one date at a time?'", "This narrows the task when several deadlines appear together.", "You can slow down deadline-heavy information.", 310, &["dates", "rescue"]),
        problem("pb_rescue_008", "Rescue: Ask for the Result First", "rescue", "toeic_600_700", "rescue", "research", "Practice saying: 'Could you tell me the result first, and then the method?'", "Result-first phrasing prevents the method from displacing the main takeaway.", "You can secure the takeaway before extra detail arrives.", 320, &["result_first", "rescue"]),
    ]
}

fn problem(
    id: &str,
    title: &str,
    mode: &str,
    level_band: &str,
    topic: &str,
    target_context: &str,
    prompt: &str,
    wm_support: &str,
    success_check: &str,
    sort_order: u32,
    tags: &[&str],
) -> ProblemRecord {
    ProblemRecord {
        id: id.to_string(),
        title: title.to_string(),
        mode: mode.to_string(),
        level_band: level_band.to_string(),
        topic: topic.to_string(),
        target_context: target_context.to_string(),
        prompt: prompt.to_string(),
        wm_support: wm_support.to_string(),
        success_check: success_check.to_string(),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        sort_order,
        source: "seeded".to_string(),
        pinned: false,
        usage_count: 0,
        success_count: 0,
        last_used_unix: 0,
        notes: String::new(),
        usage_history: Vec::new(),
    }
}

fn generated_problem(
    id: String,
    title: &str,
    mode: &str,
    level_band: &str,
    topic: &str,
    target_context: &str,
    prompt: String,
    wm_support: &str,
    success_check: &str,
    sort_order: u32,
    tags: &[&str],
) -> ProblemRecord {
    ProblemRecord {
        id,
        title: title.to_string(),
        mode: mode.to_string(),
        level_band: level_band.to_string(),
        topic: topic.to_string(),
        target_context: target_context.to_string(),
        prompt,
        wm_support: wm_support.to_string(),
        success_check: success_check.to_string(),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        sort_order,
        source: "generated".to_string(),
        pinned: false,
        usage_count: 0,
        success_count: 0,
        last_used_unix: 0,
        notes: String::new(),
        usage_history: Vec::new(),
    }
}

fn default_problem_source() -> String {
    "custom".to_string()
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

fn split_sentences(text: &str) -> Vec<String> {
    text.split(['.', '!', '?'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn summarize(sentences: &[String], normalized: &str) -> String {
    if let Some(first) = sentences.first() {
        shorten_for_summary(first)
    } else {
        shorten_for_summary(normalized)
    }
}

fn infer_topic(target_context: &str, text: &str) -> String {
    let lowered = text.to_ascii_lowercase();
    if target_context.eq_ignore_ascii_case("meeting")
        || lowered.contains("meeting")
        || lowered.contains("schedule")
        || lowered.contains("client")
    {
        "meeting".to_string()
    } else if target_context.eq_ignore_ascii_case("research")
        || lowered.contains("study")
        || lowered.contains("participants")
        || lowered.contains("result")
    {
        "research".to_string()
    } else if target_context.eq_ignore_ascii_case("self_intro") {
        "self_intro".to_string()
    } else {
        "daily".to_string()
    }
}

fn speaking_target(summary: &str) -> String {
    summary.trim_end_matches('.').to_string()
}

fn support_focus(text: &str) -> Option<String> {
    split_on_support_markers(text)
        .into_iter()
        .nth(1)
        .map(|value| value.trim().trim_end_matches('.').to_string())
        .filter(|value| !value.is_empty())
}

fn generated_id(text: &str, target_context: &str, level_band: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(target_context.as_bytes());
    hasher.update(level_band.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!("gen_{}", &digest[..12])
}

struct GeneratedProfile {
    reading_title: &'static str,
    reading_prompt: String,
    reading_support: &'static str,
    reading_success: &'static str,
    listening_title: &'static str,
    listening_prompt: String,
    listening_support: &'static str,
    listening_success: &'static str,
    speaking_title: &'static str,
    speaking_prompt: String,
    speaking_support: &'static str,
    speaking_success: &'static str,
    rescue_title: &'static str,
    rescue_prompt: String,
    rescue_support: &'static str,
    rescue_success: &'static str,
}

fn generated_profile(
    target_context: &str,
    topic: &str,
    focus_text: &str,
    support_text: &str,
    summary: &str,
) -> GeneratedProfile {
    match target_context {
        "meeting" => GeneratedProfile {
            reading_title: "Generated Decision Lock",
            reading_prompt: format!(
                "Read the core meeting point first: '{}'. Add this detail only after the decision feels stable: '{}'.",
                trim_sentence(focus_text),
                trim_sentence(support_text)
            ),
            reading_support: "Keep the decision stable before you add timing, ownership, or reason details.",
            reading_success: "You can state the decision and one follow-up detail without rereading the full line.",
            listening_title: "Generated Agenda Pause",
            listening_prompt: format!(
                "Pause after this meeting chunk and say only the checkpoint meaning: '{}'.",
                trim_sentence(focus_text)
            ),
            listening_support: "Short pauses protect the agenda or decision before the next business detail arrives.",
            listening_success: "You can repeat the meeting checkpoint before continuing.",
            speaking_title: "Generated Meeting Update",
            speaking_prompt: format!(
                "Say this as two short business steps: '{}'. Then add one short follow-up detail: '{}'.",
                speaking_target(focus_text),
                speaking_target(support_text)
            ),
            speaking_support: "Two short business steps are safer than one dense update sentence.",
            speaking_success: "You can finish the update without losing the second step.",
            rescue_title: "Generated Meeting Rescue",
            rescue_prompt: "Practice saying: 'Could you give me the decision first, and then the detail?'".to_string(),
            rescue_support: "Decision-first rescue keeps the main meeting point visible.",
            rescue_success: "You can ask for the decision quickly when the update becomes too dense.",
        },
        "research" => GeneratedProfile {
            reading_title: "Generated Research Core Lock",
            reading_prompt: format!(
                "Read the core claim first: '{}'. Treat this as support that comes second: '{}'.",
                trim_sentence(focus_text),
                trim_sentence(support_text)
            ),
            reading_support: "Hold the claim before you load method, limitation, or result detail.",
            reading_success: "You can explain the claim and one support detail in order.",
            listening_title: "Generated Result Pause",
            listening_prompt: format!(
                "Pause after the main research point and keep only this checkpoint: '{}'.",
                trim_sentence(focus_text)
            ),
            listening_support: "Research sentences often hide the takeaway behind detail, so the pause protects the result first.",
            listening_success: "You can say the main result or claim before replaying.",
            speaking_title: "Generated Research Short Units",
            speaking_prompt: format!(
                "Deliver the research point in short units: '{}'. Then add one support line: '{}'.",
                speaking_target(summary),
                speaking_target(support_text)
            ),
            speaking_support: "Short units keep the claim from collapsing under method or limitation detail.",
            speaking_success: "You can say the claim clearly and then add one supporting line.",
            rescue_title: "Generated Research Rescue",
            rescue_prompt: "Practice saying: 'Could you tell me the result first, and then the method?'".to_string(),
            rescue_support: "Result-first rescue prevents the method from displacing the main takeaway.",
            rescue_success: "You can recover the takeaway before the explanation gets longer.",
        },
        "self_intro" => GeneratedProfile {
            reading_title: "Generated Self-Intro Core Lock",
            reading_prompt: format!(
                "Read the self-introduction core first: '{}'. Add the next detail only after that feels stable.",
                trim_sentence(focus_text)
            ),
            reading_support: "One stable identity sentence is easier to hold than a full introduction at once.",
            reading_success: "You can restate the core self-introduction sentence smoothly.",
            listening_title: "Generated Self-Intro Pause",
            listening_prompt: format!(
                "Pause after this self-introduction chunk and keep only the main identity line: '{}'.",
                trim_sentence(focus_text)
            ),
            listening_support: "Pausing protects name, role, or goal before more background appears.",
            listening_success: "You can repeat the main self-introduction line without replaying.",
            speaking_title: "Generated Self-Intro Starter",
            speaking_prompt: format!(
                "Say this as two short introduction steps: '{}'. Then add one more simple detail.",
                speaking_target(summary)
            ),
            speaking_support: "A short opener lowers pressure to plan the whole self-introduction.",
            speaking_success: "You can start the introduction clearly and continue one step at a time.",
            rescue_title: "Generated Self-Intro Rescue",
            rescue_prompt: "Practice saying: 'Could I explain it one short step at a time?'".to_string(),
            rescue_support: "This makes it easier to slow the exchange before the introduction collapses.",
            rescue_success: "You can ask for a shorter interaction without losing confidence.",
        },
        _ => GeneratedProfile {
            reading_title: "Generated Core Lock",
            reading_prompt: format!(
                "Read only this first and stop before extra detail: '{}'. Then add this support second: '{}'.",
                trim_sentence(focus_text),
                trim_sentence(support_text)
            ),
            reading_support: "Keep the main sentence stable before support detail appears.",
            reading_success: "You can restate the main point in one short sentence.",
            listening_title: "Generated Pause Recall",
            listening_prompt: format!(
                "Listen to this chunk, pause, and say only the checkpoint meaning: '{}'.",
                trim_sentence(focus_text)
            ),
            listening_support: "A forced pause protects one meaning unit before the next audio arrives.",
            listening_success: "You can say the checkpoint meaning without replaying immediately.",
            speaking_title: if topic == "daily" {
                "Generated Daily Short Units"
            } else {
                "Generated Short-Unit Speaking"
            },
            speaking_prompt: format!(
                "Say this in two or three short parts instead of one long sentence: '{}'.",
                speaking_target(summary)
            ),
            speaking_support: "Short units reduce the need to plan and hold the full answer at once.",
            speaking_success: "You can finish the explanation without restarting the sentence.",
            rescue_title: "Generated Rescue Prompt",
            rescue_prompt: "Practice saying: 'Could you tell me the main point first?'".to_string(),
            rescue_support: "A rescue line creates one external anchor before more detail arrives.",
            rescue_success: "You can use the line quickly when the sentence starts to collapse.",
        },
    }
}

fn split_on_support_markers(text: &str) -> Vec<String> {
    let markers = [", but ", ", and ", " because ", " while ", " so that ", " so ", " although ", " whereas "];
    for marker in markers {
        if text.to_ascii_lowercase().contains(marker.trim()) {
            let parts = text.splitn(2, marker).map(str::trim).map(ToString::to_string).collect::<Vec<_>>();
            if parts.len() > 1 {
                return parts;
            }
        }
    }
    vec![text.to_string()]
}

fn trim_sentence(text: &str) -> String {
    text.trim().trim_end_matches('.').to_string()
}

fn shorten_for_summary(text: &str) -> String {
    let trimmed = trim_sentence(text);
    let words = trimmed.split_whitespace().collect::<Vec<_>>();
    if words.len() <= 16 {
        return trimmed;
    }
    words[..16].join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn filters_by_mode_and_query() {
        let bank = ProblemBank::seeded();
        let results = bank.list(ProblemFilter {
            mode: Some("speaking".to_string()),
            query: Some("status".to_string()),
            ..ProblemFilter::default()
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "pb_speak_002");
    }

    #[test]
    fn truncates_by_limit() {
        let bank = ProblemBank::seeded();
        let results = bank.list(ProblemFilter {
            limit: 2,
            ..ProblemFilter::default()
        });

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn generates_problem_set_from_text() {
        let bank = ProblemBank::seeded();
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The client approved the design draft, but the delivery schedule is still under review.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("meeting".to_string()),
        });

        assert_eq!(generated.items.len(), 4);
        assert_eq!(generated.topic, "meeting");
        assert!(generated.items.iter().any(|item| item.mode == "speaking"));
        let speaking = generated
            .items
            .iter()
            .find(|item| item.mode == "speaking")
            .expect("speaking item");
        assert!(speaking.prompt.contains("two short business steps"));
        let rescue = generated
            .items
            .iter()
            .find(|item| item.mode == "rescue")
            .expect("rescue item");
        assert!(rescue.prompt.contains("decision first"));
    }

    #[test]
    fn generates_research_specific_wording() {
        let bank = ProblemBank::seeded();
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The study found lower overload, but live conversation data is still limited.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("research".to_string()),
        });

        let reading = generated
            .items
            .iter()
            .find(|item| item.mode == "reading")
            .expect("reading item");
        assert!(reading.prompt.contains("core claim"));
        let rescue = generated
            .items
            .iter()
            .find(|item| item.mode == "rescue")
            .expect("rescue item");
        assert!(rescue.prompt.contains("result first"));
    }

    #[test]
    fn generates_self_intro_specific_wording() {
        let bank = ProblemBank::seeded();
        let generated = bank.generate(ProblemGenerationRequest {
            text: "Hello, I support students who need lower-load English practice, and I focus on step-by-step communication.".to_string(),
            level_band: Some("starter".to_string()),
            topic: None,
            target_context: Some("self_intro".to_string()),
        });

        let speaking = generated
            .items
            .iter()
            .find(|item| item.mode == "speaking")
            .expect("speaking item");
        assert!(speaking.prompt.contains("introduction steps"));
        let rescue = generated
            .items
            .iter()
            .find(|item| item.mode == "rescue")
            .expect("rescue item");
        assert!(rescue.prompt.contains("one short step at a time"));
    }

    #[test]
    fn shortens_long_summary() {
        let bank = ProblemBank::seeded();
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The client approved the first draft after the review meeting, and the operations team will send the updated shipping timeline after checking supplier availability tomorrow morning.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("meeting".to_string()),
        });

        let summary_word_count = generated.summary.split_whitespace().count();
        assert!(summary_word_count <= 16);
        assert!(generated.summary.contains("client approved"));
    }

    #[test]
    fn saves_generated_items_to_custom_store() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The client approved the design draft, but the delivery schedule is still under review.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("meeting".to_string()),
        });

        let saved = bank
            .save_generated_set(&generated, ProblemSaveSource::Generated)
            .expect("save generated set");

        assert_eq!(saved.saved_count, 4);
        assert!(saved.total_custom >= 4);
        assert!(temp_path.exists());
        let persisted = fs::read_to_string(&temp_path).expect("persisted problem bank");
        assert!(persisted.contains("\"saved_"));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn reports_stats_with_custom_items() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The study found lower overload, but live conversation data is still limited.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("research".to_string()),
        });
        bank.save_generated_set(&generated, ProblemSaveSource::Reviewed)
            .expect("save reviewed set");

        let stats = bank.stats();
        assert!(stats.total >= stats.seeded);
        assert!(stats.custom >= 4);
        assert!(stats.by_mode.contains_key("reading"));
        assert!(stats.by_context.contains_key("research"));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn recommends_matching_items_first() {
        let bank = ProblemBank::seeded();
        let items = bank.recommend(ProblemRecommendationRequest {
            preferred_mode: Some("speaking".to_string()),
            target_context: Some("meeting".to_string()),
            level_band: Some("toeic_750_800".to_string()),
            topic: Some("meeting".to_string()),
            focus_tag: Some("status_update".to_string()),
            prefer_review: false,
            avoid_mastered: false,
            limit: 3,
        });

        assert!(!items.is_empty());
        assert_eq!(items[0].id, "pb_speak_002");
    }

    #[test]
    fn deletes_custom_item() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The client approved the design draft, but the delivery schedule is still under review.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("meeting".to_string()),
        });
        let saved = bank
            .save_generated_set(&generated, ProblemSaveSource::Generated)
            .expect("save generated set");

        let deleted = bank
            .delete_custom(&saved.items[0].id)
            .expect("delete saved problem");

        assert_eq!(deleted.id, saved.items[0].id);
        assert!(bank.get(&saved.items[0].id).is_none());
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn updates_custom_item_metadata() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The client approved the design draft, but the delivery schedule is still under review.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("meeting".to_string()),
        });
        let saved = bank
            .save_generated_set(&generated, ProblemSaveSource::Reviewed)
            .expect("save reviewed set");

        let updated = bank
            .update_custom(
                &saved.items[0].id,
                ProblemRecordUpdate {
                    title: Some("Pinned custom problem".to_string()),
                    prompt: None,
                    wm_support: None,
                    success_check: None,
                    tags: None,
                    notes: Some("strong for meeting rehearsal".to_string()),
                    pinned: Some(true),
                },
            )
            .expect("update custom");

        assert_eq!(updated.title, "Pinned custom problem");
        assert!(updated.pinned);
        assert!(updated.notes.contains("meeting rehearsal"));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn records_usage_on_custom_item() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let generated = bank.generate(ProblemGenerationRequest {
            text: "The study found lower overload, but live conversation data is still limited.".to_string(),
            level_band: Some("toeic_750_800".to_string()),
            topic: None,
            target_context: Some("research".to_string()),
        });
        let saved = bank
            .save_generated_set(&generated, ProblemSaveSource::Generated)
            .expect("save generated set");

        let updated = bank
            .record_usage(
                &saved.items[0].id,
                ProblemUsageEvent {
                    successful: true,
                    occurred_at_unix: Some(123456789),
                    append_note: Some("worked well in a short session".to_string()),
                },
            )
            .expect("record usage");

        assert_eq!(updated.usage_count, 1);
        assert_eq!(updated.success_count, 1);
        assert_eq!(updated.last_used_unix, 123456789);
        assert!(updated.notes.contains("worked well"));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn list_custom_can_filter_by_source_and_pinned() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let saved = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone seed problem");

        bank.update_custom(
            &saved.items[0].id,
            ProblemRecordUpdate {
                title: None,
                prompt: None,
                wm_support: None,
                success_check: None,
                tags: None,
                notes: None,
                pinned: Some(true),
            },
        )
        .expect("pin custom problem");

        let results = bank.list_custom(ProblemFilter {
            source: Some("reviewed".to_string()),
            pinned_only: true,
            ..ProblemFilter::default()
        });

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].source, "reviewed");
        assert!(results[0].pinned);
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn activity_returns_recent_entries_in_descending_order() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let saved = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone seed problem");
        let saved_id = saved.items[0].id.clone();

        bank.record_usage(
            &saved_id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(100),
                append_note: Some("first try".to_string()),
            },
        )
        .expect("record first usage");
        bank.record_usage(
            &saved_id,
            ProblemUsageEvent {
                successful: true,
                occurred_at_unix: Some(200),
                append_note: Some("second try".to_string()),
            },
        )
        .expect("record second usage");

        let activity = bank.activity(ProblemActivityRequest {
            successful: Some(true),
            ..ProblemActivityRequest::default()
        });

        assert_eq!(activity.len(), 1);
        assert_eq!(activity[0].problem_id, saved_id);
        assert_eq!(activity[0].occurred_at_unix, 200);
        assert!(activity[0].successful);
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn insights_summarize_success_and_top_used_problems() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let saved = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone seed problem");
        let saved_id = saved.items[0].id.clone();

        bank.record_usage(
            &saved_id,
            ProblemUsageEvent {
                successful: true,
                occurred_at_unix: Some(100),
                append_note: Some("first success".to_string()),
            },
        )
        .expect("record first usage");
        bank.record_usage(
            &saved_id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(200),
                append_note: Some("second try".to_string()),
            },
        )
        .expect("record second usage");

        let insights = bank.insights(ProblemActivityRequest {
            source: Some("reviewed".to_string()),
            ..ProblemActivityRequest::default()
        });

        assert_eq!(insights.total_history_entries, 2);
        assert_eq!(insights.successful_history_entries, 1);
        assert_eq!(insights.failed_history_entries, 1);
        assert_eq!(insights.top_used_problems.len(), 1);
        assert_eq!(insights.top_used_problems[0].problem_id, saved_id);
        assert_eq!(insights.top_used_problems[0].usage_count, 2);
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn review_queue_prioritizes_recent_failures_over_mastered_items() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());

        let struggling = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone struggling problem");
        let mastered = bank
            .clone_problem("pb_read_001", ProblemSaveSource::Reviewed)
            .expect("clone mastered problem");

        let struggling_id = struggling.items[0].id.clone();
        let mastered_id = mastered.items[0].id.clone();

        bank.record_usage(
            &struggling_id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(400),
                append_note: Some("lost the second step".to_string()),
            },
        )
        .expect("record struggling usage");

        for occurred_at_unix in [100_u64, 200_u64, 300_u64] {
            bank.record_usage(
                &mastered_id,
                ProblemUsageEvent {
                    successful: true,
                    occurred_at_unix: Some(occurred_at_unix),
                    append_note: Some("stable".to_string()),
                },
            )
            .expect("record mastered usage");
        }

        let queue = bank.review_queue(ProblemRecommendationRequest {
            preferred_mode: None,
            target_context: None,
            level_band: None,
            topic: None,
            focus_tag: None,
            prefer_review: true,
            avoid_mastered: true,
            limit: 5,
        });

        assert!(!queue.is_empty());
        assert_eq!(queue[0].id, struggling_id);
        assert!(!queue.iter().any(|item| item.id == mastered_id));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn weakness_queue_groups_review_candidates_by_mode() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());

        let speaking = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone speaking problem");
        let reading = bank
            .clone_problem("pb_read_001", ProblemSaveSource::Reviewed)
            .expect("clone reading problem");

        bank.record_usage(
            &speaking.items[0].id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(700),
                append_note: Some("speaking collapsed".to_string()),
            },
        )
        .expect("record speaking usage");
        bank.record_usage(
            &reading.items[0].id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(800),
                append_note: Some("lost the clause".to_string()),
            },
        )
        .expect("record reading usage");

        let queue = bank.weakness_queue(ProblemRecommendationRequest {
            preferred_mode: None,
            target_context: None,
            level_band: None,
            topic: None,
            focus_tag: None,
            prefer_review: true,
            avoid_mastered: true,
            limit: 3,
        });

        assert!(queue.groups.iter().any(|group| group.mode == "speaking"));
        assert!(queue.groups.iter().any(|group| group.mode == "reading"));
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn dashboard_combines_stats_insights_and_queues() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());
        let saved = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone seed problem");

        bank.record_usage(
            &saved.items[0].id,
            ProblemUsageEvent {
                successful: false,
                occurred_at_unix: Some(900),
                append_note: Some("dashboard failure".to_string()),
            },
        )
        .expect("record usage");

        let dashboard = bank.dashboard(
            ProblemRecommendationRequest {
                preferred_mode: Some("speaking".to_string()),
                target_context: None,
                level_band: None,
                topic: None,
                focus_tag: None,
                prefer_review: true,
                avoid_mastered: true,
                limit: 3,
            },
            ProblemActivityRequest {
                source: Some("reviewed".to_string()),
                ..ProblemActivityRequest::default()
            },
        );

        assert!(dashboard.stats.custom >= 1);
        assert!(dashboard.insights.total_history_entries >= 1);
        assert!(!dashboard.review_queue.is_empty());
        assert!(
            dashboard
                .weakness_queue
                .groups
                .iter()
                .any(|group| group.mode == "speaking")
        );
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn clones_seed_problem_into_custom_store() {
        let temp_path = temp_problem_bank_path();
        let bank = ProblemBank::with_persisted_path(temp_path.clone());

        let saved = bank
            .clone_problem("pb_speak_002", ProblemSaveSource::Reviewed)
            .expect("clone seed problem");

        assert_eq!(saved.saved_count, 1);
        assert_eq!(saved.items[0].title, "Two-Step Link: Status Update");
        assert!(saved.items[0].tags.iter().any(|tag| tag == "saved"));
        let _ = fs::remove_file(temp_path);
    }

    fn temp_problem_bank_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("mse-proxy-problem-bank-{unique}.json"))
    }
}
