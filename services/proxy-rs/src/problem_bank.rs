use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

#[derive(Clone)]
pub struct ProblemBank {
    items: Arc<Vec<ProblemRecord>>,
    by_id: Arc<HashMap<String, ProblemRecord>>,
}

impl ProblemBank {
    pub fn seeded() -> Self {
        let items = seeded_records();
        let by_id = items
            .iter()
            .cloned()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();

        Self {
            items: Arc::new(items),
            by_id: Arc::new(by_id),
        }
    }

    pub fn list(&self, filter: ProblemFilter) -> Vec<ProblemRecord> {
        let mut matched = self
            .items
            .iter()
            .filter(|item| filter.matches(item))
            .cloned()
            .collect::<Vec<_>>();
        matched.sort_by(|a, b| a.sort_order.cmp(&b.sort_order).then_with(|| a.id.cmp(&b.id)));
        if matched.len() > filter.limit {
            matched.truncate(filter.limit);
        }
        matched
    }

    pub fn get(&self, id: &str) -> Option<ProblemRecord> {
        self.by_id.get(id).cloned()
    }
}

#[derive(Clone, Debug, Serialize)]
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
}

#[derive(Clone, Debug)]
pub struct ProblemFilter {
    pub mode: Option<String>,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
