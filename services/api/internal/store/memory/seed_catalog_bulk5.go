package memory

import (
	"time"

	"memory-safe-english/services/api/internal/domain"
)

func additionalSeedCatalogBulk5(now time.Time) []domain.Content {
	specs := []seedSpec{
		{"cnt_self_intro_029", "Self Introduction: Presentation Goal", "reading", "intro", "self_intro", "When I present in English, my goal is to stay clear and avoid losing the main sentence halfway through.", "Presentation goal in one sentence"},
		{"cnt_self_intro_030", "Self Introduction: Short Closing", "speaking_template", "intro", "self_intro", "That is my short introduction. Thank you for listening.", "Short closing for introductions"},
		{"cnt_daily_045", "Daily Reading: Ask for the Main Entrance", "reading", "intro", "daily", "Could you tell me which entrance is best if I only need the information desk?", "Main-entrance question"},
		{"cnt_daily_046", "Daily Listening: Short Safety Notice", "listening", "intro", "daily", "Please keep your bag with you and move away from the door after boarding.", "Safety notice with two short actions"},
		{"cnt_daily_047", "Daily Speaking: Ask for the Topic First", "speaking_template", "intro", "daily", "Before the details, could you tell me the topic first?", "Topic-first speaking template"},
		{"cnt_research_047", "Research Reading: Why UI Matters", "reading", "intermediate", "research", "Interface design matters because overload can appear even when the learner already knows the vocabulary.", "Why UI matters in one sentence"},
		{"cnt_research_048", "Research Listening: Main Limitation", "listening", "intermediate", "research", "The main limitation is that we still need evidence from live conversation, not only controlled reading tasks.", "Main limitation for listening practice"},
		{"cnt_research_049", "Research Speaking: Core Takeaway", "speaking_template", "intermediate", "research", "The core takeaway is simple. Lower holding demand helps understanding.", "Core takeaway in short units"},
		{"cnt_research_050", "Research Reading: User Benefit", "reading", "intermediate", "research", "Users benefit when the system protects the core relation before extra detail starts to compete for attention.", "User benefit statement"},
		{"cnt_meeting_041", "Meeting Reading: Ask for One Priority", "reading", "intermediate", "meeting", "If we choose only one priority for this week, which priority should it be?", "One-priority meeting question"},
		{"cnt_meeting_042", "Meeting Listening: Confirm the Scope", "listening", "intermediate", "meeting", "To confirm the scope, today we will review only the first two screens and not the full onboarding flow.", "Scope confirmation for meetings"},
		{"cnt_meeting_043", "Meeting Speaking: Short Progress Update", "speaking_template", "intermediate", "meeting", "Progress is steady. The main flow works. The final check is still open.", "Short progress update"},
		{"cnt_meeting_044", "Meeting Reading: Ask for One Example", "reading", "intermediate", "meeting", "Could you give one example of the user problem before we discuss the full report?", "Example-first meeting question"},
		{"cnt_rescue_043", "Rescue: Ask for the Short Version First", "rescue", "intro", "rescue", "Could you give me the short version first?", "Rescue phrase for short-version-first"},
		{"cnt_rescue_044", "Rescue: Ask for the Core Idea Again", "rescue", "intro", "rescue", "Could you say the core idea again?", "Rescue phrase for core idea repetition"},
		{"cnt_rescue_045", "Rescue: Ask for the Main Task", "rescue", "intro", "rescue", "What is the main task for me?", "Rescue phrase for main task"},
		{"cnt_rescue_046", "Rescue: Ask to Hold the Last Sentence", "rescue", "intro", "rescue", "Can we keep the last sentence visible for a moment?", "Rescue phrase for visual sentence hold"},
		{"cnt_rescue_047", "Rescue: Ask for One More Pause", "rescue", "intro", "rescue", "Could you pause once more before the next point?", "Rescue phrase for extra pause"},
		{"cnt_rescue_048", "Rescue: Ask for a Simpler Explanation", "rescue", "intro", "rescue", "Could you explain it in a simpler way first?", "Rescue phrase for simpler explanation"},
		{"cnt_rescue_049", "Rescue: Ask for the First Step Again", "rescue", "intro", "rescue", "Could you repeat the first step one more time?", "Rescue phrase for first-step repetition"},
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
