use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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

    pub fn generate(&self, request: ProblemGenerationRequest) -> GeneratedProblemSet {
        let normalized = normalize_text(&request.text);
        let sentences = split_sentences(&normalized);
        let summary = summarize(&sentences, &normalized);
        let focus_text = sentences
            .first()
            .cloned()
            .unwrap_or_else(|| normalized.clone());
        let level_band = request
            .level_band
            .unwrap_or_else(|| "toeic_600_700".to_string());
        let target_context = request
            .target_context
            .unwrap_or_else(|| "general".to_string());
        let topic = request.topic.unwrap_or_else(|| infer_topic(&target_context, &normalized));
        let base_id = generated_id(&normalized, &target_context, &level_band);

        let items = vec![
            generated_problem(
                format!("{base_id}_read"),
                "Generated Core Lock",
                "reading",
                &level_band,
                &topic,
                &target_context,
                format!("Read only this first and stop before extra detail: '{focus_text}'"),
                "Keep the main sentence stable before support detail appears.",
                "You can restate the main point in one short sentence.",
                10,
                &["generated", "core_lock"],
            ),
            generated_problem(
                format!("{base_id}_listen"),
                "Generated Pause Recall",
                "listening",
                &level_band,
                &topic,
                &target_context,
                format!("Listen to this chunk, pause, and say only the checkpoint meaning: '{focus_text}'"),
                "A forced pause protects one meaning unit before the next audio arrives.",
                "You can say the checkpoint meaning without replaying immediately.",
                20,
                &["generated", "pause_recall"],
            ),
            generated_problem(
                format!("{base_id}_speak"),
                "Generated Short-Unit Speaking",
                "speaking",
                &level_band,
                &topic,
                &target_context,
                format!(
                    "Say this in two or three short parts instead of one long sentence: '{}'",
                    speaking_target(&summary)
                ),
                "Short units reduce the need to plan and hold the full answer at once.",
                "You can finish the explanation without restarting the sentence.",
                30,
                &["generated", "short_unit"],
            ),
            generated_problem(
                format!("{base_id}_rescue"),
                "Generated Rescue Prompt",
                "rescue",
                &level_band,
                "rescue",
                &target_context,
                rescue_prompt(&target_context),
                "A rescue line creates one external anchor before more detail arrives.",
                "You can use the line quickly when the sentence starts to collapse.",
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProblemGenerationRequest {
    pub text: String,
    pub level_band: Option<String>,
    pub topic: Option<String>,
    pub target_context: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GeneratedProblemSet {
    pub source_text: String,
    pub summary: String,
    pub target_context: String,
    pub level_band: String,
    pub topic: String,
    pub items: Vec<ProblemRecord>,
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
    }
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
        first.clone()
    } else {
        normalized.to_string()
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

fn rescue_prompt(target_context: &str) -> String {
    match target_context {
        "meeting" => "Practice saying: 'Could you give me the decision first, and then the detail?'".to_string(),
        "research" => "Practice saying: 'Could you repeat the main result first?'".to_string(),
        "self_intro" => "Practice saying: 'Could I explain it one short step at a time?'".to_string(),
        _ => "Practice saying: 'Could you tell me the main point first?'".to_string(),
    }
}

fn generated_id(text: &str, target_context: &str, level_band: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.update(target_context.as_bytes());
    hasher.update(level_band.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!("gen_{}", &digest[..12])
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
    }
}
