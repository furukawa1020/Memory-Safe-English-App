package memory

import (
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func additionalSeedCatalogBulk4(now time.Time) []domain.Content {
	specs := []seedSpec{
		{"cnt_self_intro_025", "Self Introduction: Current Project", "reading", "intro", "self_intro", "Right now, my project is about helping learners keep the main point visible during difficult English tasks.", "Current project in one sentence"},
		{"cnt_self_intro_026", "Self Introduction: Small Success", "reading", "intro", "self_intro", "A small success for me was giving a short English summary without losing the sentence halfway through.", "Small success statement"},
		{"cnt_self_intro_027", "Self Introduction: Calm Start", "speaking_template", "intro", "self_intro", "I will start simply. My name is Emi. My topic is overload-friendly English support.", "Calm-start self introduction"},
		{"cnt_self_intro_028", "Self Introduction: Listening Habit", "listening", "intro", "self_intro", "When a talk becomes dense, I try to keep the topic and the conclusion instead of every small detail.", "Listening habit for gist retention"},
		{"cnt_daily_039", "Daily Reading: Ask for the Main Rule", "reading", "intro", "daily", "I do not need every rule right now. Could you tell me the main rule first?", "Daily rule question with narrowed scope"},
		{"cnt_daily_040", "Daily Reading: Ask for the First Document", "reading", "intro", "daily", "Which document should I prepare first if I want to finish this registration today?", "Document-priority question"},
		{"cnt_daily_041", "Daily Listening: Number Ticket Notice", "listening", "intro", "daily", "Customers with tickets from A twenty to A twenty-five should move to desk three now.", "Queue notice with one destination"},
		{"cnt_daily_042", "Daily Listening: Delivery Delay", "listening", "intro", "daily", "Your package will arrive tomorrow morning instead of tonight because of the weather conditions.", "Delivery delay notice"},
		{"cnt_daily_043", "Daily Speaking: Ask for the Main Option", "speaking_template", "intro", "daily", "Could you show me the main option first? I can compare the others after that.", "Option-first request in short units"},
		{"cnt_daily_044", "Daily Speaking: Explain Confusion", "speaking_template", "intro", "daily", "I understand the first step, but the second step is confusing for me.", "Short confusion explanation"},
		{"cnt_research_041", "Research Reading: Main Hypothesis", "reading", "intermediate", "research", "Our hypothesis was that learners would understand more when the interface reduced holding pressure before integration.", "Main hypothesis statement"},
		{"cnt_research_042", "Research Reading: Summary Task", "reading", "intermediate", "research", "After each passage, learners produced a short summary so we could compare preserved meaning across conditions.", "Summary task description"},
		{"cnt_research_043", "Research Listening: Design Rationale", "listening", "intermediate", "research", "We used short pauses because they create an external checkpoint before the next detail increases the load.", "Design rationale for pause use"},
		{"cnt_research_044", "Research Listening: Practical Implication", "listening", "intermediate", "research", "The practical implication is that better pacing may help even before vocabulary size changes.", "Practical implication in one sentence"},
		{"cnt_research_045", "Research Speaking: Explain the Measure", "speaking_template", "intermediate", "research", "We used one simple measure. We asked for the main point. Then we checked overload ratings.", "Measure explanation in short steps"},
		{"cnt_research_046", "Research Speaking: Explain the Benefit", "speaking_template", "intermediate", "research", "The benefit is clear. Learners stay with the sentence longer. That makes integration easier.", "Benefit explanation in short units"},
		{"cnt_meeting_035", "Meeting Reading: Confirm the Deadline", "reading", "intermediate", "meeting", "Before we discuss details, can we confirm whether the deadline is still next Wednesday afternoon?", "Deadline confirmation question"},
		{"cnt_meeting_036", "Meeting Reading: Ask for the Main Risk Only", "reading", "intermediate", "meeting", "If we focus on only one risk today, which risk should come first?", "Single-risk prioritization question"},
		{"cnt_meeting_037", "Meeting Listening: Keep the Scope Small", "listening", "intermediate", "meeting", "For today's meeting, let us keep the scope small and decide only the first release requirement.", "Scope-limiting meeting cue"},
		{"cnt_meeting_038", "Meeting Listening: One Follow-up Task", "listening", "intermediate", "meeting", "After the meeting, please send one follow-up task that you can finish by tomorrow noon.", "Follow-up instruction with one task"},
		{"cnt_meeting_039", "Meeting Speaking: Ask for Confirmation", "speaking_template", "intermediate", "meeting", "I want to confirm one point. Is the main priority still stability over speed?", "Confirmation template for meetings"},
		{"cnt_meeting_040", "Meeting Speaking: Suggest a Simpler Plan", "speaking_template", "intermediate", "meeting", "I suggest a simpler plan. We ship the core part now. The extra part can wait.", "Simpler-plan suggestion"},
		{"cnt_rescue_035", "Rescue: Ask for the First Point Again", "rescue", "intro", "rescue", "Could you say the first point again?", "Rescue phrase for first-point recovery"},
		{"cnt_rescue_036", "Rescue: Ask for the Core Sentence", "rescue", "intro", "rescue", "Can you give me the core sentence first?", "Rescue phrase for core sentence"},
		{"cnt_rescue_037", "Rescue: Ask for a Shorter Answer", "rescue", "intro", "rescue", "Could you make the answer a little shorter?", "Rescue phrase for shorter answers"},
		{"cnt_rescue_038", "Rescue: Ask for One Decision", "rescue", "intro", "rescue", "What is the one decision we made?", "Rescue phrase for decision focus"},
		{"cnt_rescue_039", "Rescue: Ask for One Task", "rescue", "intro", "rescue", "What is the one task for me now?", "Rescue phrase for single-task focus"},
		{"cnt_rescue_040", "Rescue: Ask to Pause Before Details", "rescue", "intro", "rescue", "Please pause before the details and tell me the topic first.", "Rescue phrase for topic-first pacing"},
		{"cnt_rescue_041", "Rescue: Ask for Two Short Parts", "rescue", "intro", "rescue", "Can you split that into two short parts?", "Rescue phrase for splitting explanations"},
		{"cnt_rescue_042", "Rescue: Ask for the Keyword Again", "rescue", "intro", "rescue", "Could you repeat the keyword one more time?", "Rescue phrase for keyword repetition"},
	}

	items := make([]domain.Content, 0, len(specs))
	for _, spec := range specs {
		items = append(items, domain.Content{
			ID:          spec.id,
			Title:       spec.title,
			ContentType: spec.contentType,
			Level:       spec.level,
			Topic:       spec.topic,
			Language:    "en",
			RawText:     spec.rawText,
			SummaryText: spec.summaryText,
			CreatedAt:   now,
			UpdatedAt:   now,
		})
	}

	return items
}
