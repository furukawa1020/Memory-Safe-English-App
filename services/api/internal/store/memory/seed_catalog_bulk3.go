package memory

import (
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func additionalSeedCatalogBulk3(now time.Time) []domain.Content {
	specs := []seedSpec{
		{"cnt_self_intro_021", "Self Introduction: Research Theme", "reading", "intro", "self_intro", "My current theme is simple sentence support for learners who lose track when too much information arrives together.", "Research theme in one sentence"},
		{"cnt_self_intro_022", "Self Introduction: Speaking Preference", "reading", "intro", "self_intro", "I prefer short speaking turns because I can stay accurate when I build one unit at a time.", "Speaking preference statement"},
		{"cnt_self_intro_023", "Self Introduction: Short Greeting", "speaking_template", "intro", "self_intro", "Hello. I am Nao. I work on learning support tools.", "Very short greeting template"},
		{"cnt_self_intro_024", "Self Introduction: Listening Snapshot Two", "listening", "intro", "self_intro", "I can usually follow the first point, but I need pauses when the explanation changes direction quickly.", "Listening snapshot with one need"},
		{"cnt_daily_033", "Daily Reading: Ask for a Quiet Space", "reading", "intro", "daily", "Is there a quieter place where I can read this form before I answer the questions?", "Quiet-space request with one purpose"},
		{"cnt_daily_034", "Daily Reading: Confirm the Price", "reading", "intro", "daily", "Before I pay, could you confirm whether this price already includes the service fee?", "Price confirmation question"},
		{"cnt_daily_035", "Daily Listening: Counter Number", "listening", "intro", "daily", "For ticket changes, please go to counter four near the elevator.", "Simple direction to one counter"},
		{"cnt_daily_036", "Daily Listening: Class Starts Late", "listening", "intro", "daily", "Today's class will start ten minutes late, so please wait outside room 204.", "Short schedule change for class"},
		{"cnt_daily_037", "Daily Speaking: Ask for One Example", "speaking_template", "intro", "daily", "I need one example first. After that, I can understand the rest more easily.", "Speaking template for example-first support"},
		{"cnt_daily_038", "Daily Speaking: Confirm the Time Again", "speaking_template", "intro", "daily", "Could we confirm the time again? I wrote two o'clock, but I want to be sure.", "Time confirmation in short units"},
		{"cnt_research_035", "Research Reading: Why Chunking Helps", "reading", "intermediate", "research", "Chunking helps because the learner can stabilize one meaningful unit before the next modifier adds extra load.", "Why chunking helps in one causal sentence"},
		{"cnt_research_036", "Research Reading: Why Skeleton Helps", "reading", "intermediate", "research", "A skeleton view helps by keeping the main relation visible even when many details are temporarily hidden.", "Why skeleton helps in one sentence"},
		{"cnt_research_037", "Research Listening: Main Interpretation", "listening", "intermediate", "research", "The main interpretation is that comprehension can improve when holding demand drops, even without extra vocabulary study.", "Interpretation of the main result"},
		{"cnt_research_038", "Research Listening: Limitation Reminder", "listening", "intermediate", "research", "We still need to test spontaneous conversation because the current study focused on controlled sentence inputs.", "Limitation reminder with one future need"},
		{"cnt_research_039", "Research Speaking: Explain the Interface", "speaking_template", "intermediate", "research", "The interface is simple. It highlights the core first. Then it adds support in smaller steps.", "Interface explanation in short units"},
		{"cnt_research_040", "Research Speaking: Explain the Learner Need", "speaking_template", "intermediate", "research", "The learner need is clear. Known words are not enough. Stable integration is the harder part.", "Learner need explanation for speaking"},
		{"cnt_meeting_029", "Meeting Reading: Confirm the Order", "reading", "intermediate", "meeting", "Could we confirm the order first: summary, risk, and then next action?", "Meeting order confirmation"},
		{"cnt_meeting_030", "Meeting Reading: Ask for the Main Update", "reading", "intermediate", "meeting", "Before the detailed explanation, what is the main update from yesterday?", "Main-update question for meetings"},
		{"cnt_meeting_031", "Meeting Listening: Decide One Owner", "listening", "intermediate", "meeting", "Let us choose one owner for this task today so the next step stays clear.", "Listening sentence about choosing one owner"},
		{"cnt_meeting_032", "Meeting Listening: Review One Risk", "listening", "intermediate", "meeting", "Please review one risk first, and then we can decide whether more changes are necessary.", "Review-one-risk instruction"},
		{"cnt_meeting_033", "Meeting Speaking: State the Core Issue", "speaking_template", "intermediate", "meeting", "The core issue is simple. The wording is dense. New users lose the topic too fast.", "Core-issue speaking template"},
		{"cnt_meeting_034", "Meeting Speaking: Restate the Decision", "speaking_template", "intermediate", "meeting", "Let me restate the decision. We keep the release date. We reduce the scope.", "Decision restatement template"},
		{"cnt_rescue_027", "Rescue: Ask for the Main Topic", "rescue", "intro", "rescue", "Could you say the main topic first?", "Rescue phrase for topic-first support"},
		{"cnt_rescue_028", "Rescue: Ask to Slow the Pace", "rescue", "intro", "rescue", "Could we slow the pace a little?", "Rescue phrase for slower pacing"},
		{"cnt_rescue_029", "Rescue: Ask for a Short Summary", "rescue", "intro", "rescue", "Can you give me a short summary before the details?", "Rescue phrase for summary-first support"},
		{"cnt_rescue_030", "Rescue: Ask to Keep the Core Visible", "rescue", "intro", "rescue", "Can we keep the core sentence visible while you explain?", "Rescue phrase for visual core support"},
		{"cnt_rescue_031", "Rescue: Ask What I Should Do", "rescue", "intro", "rescue", "What should I do first?", "Rescue phrase for first action"},
		{"cnt_rescue_032", "Rescue: Ask to Repeat the Conclusion", "rescue", "intro", "rescue", "Could you repeat the conclusion one more time?", "Rescue phrase for conclusion repetition"},
		{"cnt_rescue_033", "Rescue: Ask for a Simpler Step", "rescue", "intro", "rescue", "Is there a simpler step to start with?", "Rescue phrase for easier first step"},
		{"cnt_rescue_034", "Rescue: Ask for One Keyword", "rescue", "intro", "rescue", "Could you give me one keyword to remember?", "Rescue phrase for one-keyword anchor"},
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
