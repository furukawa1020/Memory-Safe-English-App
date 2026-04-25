package memory

import (
	"time"

	"memory-safe-english/services/api/internal/domain"
)

type seedSpec struct {
	id          string
	title       string
	contentType string
	level       string
	topic       string
	rawText     string
	summaryText string
}

func additionalSeedCatalogBulk(now time.Time) []domain.Content {
	specs := []seedSpec{
		{"cnt_self_intro_013", "Self Introduction: Study Style", "reading", "intro", "self_intro", "I work best when I can focus on one idea at a time and check the previous point whenever I need to.", "Study style statement for low-load reading"},
		{"cnt_self_intro_014", "Self Introduction: Short Lab Update", "speaking_template", "intro", "self_intro", "This week was simple. I read two papers. I also prepared one short survey.", "Short lab update in stable units"},
		{"cnt_self_intro_015", "Self Introduction: Listening Preference", "reading", "intro", "self_intro", "When people speak quickly, I often ask for the conclusion first and the details after that.", "Preference statement for conclusion-first listening"},
		{"cnt_self_intro_016", "Self Introduction: Classroom Version", "listening", "intro", "self_intro", "Hi, I am Miki. I am interested in communication support, especially for students who feel overloaded in class.", "Classroom-style self introduction"},
		{"cnt_daily_019", "Daily Reading: Ask for Wi-Fi Help", "reading", "intro", "daily", "I connected to the network, but the login page did not open. Could you help me check the next step?", "Wi-Fi help request with one problem"},
		{"cnt_daily_020", "Daily Reading: Food Allergy Check", "reading", "intro", "daily", "Does this meal contain nuts, or is there a safer option without them?", "Food allergy question with two choices"},
		{"cnt_daily_021", "Daily Listening: Boarding Call", "listening", "intro", "daily", "Passengers in group three may now board through gate eleven with passports ready.", "Boarding call with one action"},
		{"cnt_daily_022", "Daily Listening: Last Order", "listening", "intro", "daily", "This is the last order for hot meals, so please decide before the kitchen closes in five minutes.", "Restaurant announcement with one deadline"},
		{"cnt_daily_023", "Daily Speaking: Ask for the First Step", "speaking_template", "intro", "daily", "I am a little lost. Could you tell me the first step only? I can follow after that.", "First-step request in short units"},
		{"cnt_daily_024", "Daily Speaking: Explain a Small Problem", "speaking_template", "intro", "daily", "The screen is open, but the button does not respond. I tried again, and nothing changed.", "Short problem report for support counters"},
		{"cnt_daily_025", "Daily Reading: Ask for a Quieter Seat", "reading", "intro", "daily", "If possible, could I move to a quieter seat because I need to concentrate for a while?", "Seat-change request with one reason"},
		{"cnt_daily_026", "Daily Listening: Package Pickup", "listening", "intro", "daily", "Your package is ready for pickup at locker twelve, and the code will remain active until eight tonight.", "Pickup instruction with one deadline"},
		{"cnt_research_021", "Research Reading: Study Question", "reading", "intermediate", "research", "Our main question was whether reducing holding demand would help learners integrate known words into one stable sentence meaning.", "Research question with one causal focus"},
		{"cnt_research_022", "Research Reading: Comparison Detail", "reading", "intermediate", "research", "The control condition showed the full sentence at once, while the support condition highlighted one core chunk before the modifiers appeared.", "Comparison between two conditions"},
		{"cnt_research_023", "Research Listening: Speaker Summary", "listening", "intermediate", "research", "In short, the interface did not teach more vocabulary, but it did help learners keep the sentence long enough to understand it.", "Short oral summary of the main result"},
		{"cnt_research_024", "Research Listening: Measurement Reminder", "listening", "intermediate", "research", "After each passage, participants rated overload, repeated the main point, and noted where they lost track of the sentence.", "Measurement reminder with three actions"},
		{"cnt_research_025", "Research Speaking: Motivation in Steps", "speaking_template", "intermediate", "research", "The motivation is practical. Many learners know the words. They still lose the sentence when details pile up.", "Motivation template for research speaking"},
		{"cnt_research_026", "Research Speaking: Result in Steps", "speaking_template", "intermediate", "research", "The result has two parts. Overload went down. Summary quality went up.", "Result template using two short claims"},
		{"cnt_research_027", "Research Reading: Design Principle", "reading", "intermediate", "research", "The design principle was simple: do not force the learner to carry every detail while the main meaning is still unstable.", "Design principle in one sentence"},
		{"cnt_research_028", "Research Reading: Broader Impact", "reading", "intermediate", "research", "This approach may also help meetings and lectures because the same overload problem appears outside language classes.", "Broader impact statement"},
		{"cnt_meeting_017", "Meeting Reading: Ask for Today's Goal", "reading", "intermediate", "meeting", "Before we continue, could someone restate today's goal in one short sentence?", "Meeting question for goal reset"},
		{"cnt_meeting_018", "Meeting Reading: Confirm Responsibility", "reading", "intermediate", "meeting", "Just to confirm, are you responsible for the slides, and am I responsible for the final notes?", "Responsibility confirmation question"},
		{"cnt_meeting_019", "Meeting Listening: Review the Draft", "listening", "intermediate", "meeting", "Please read the draft once tonight and mark only the parts that block understanding.", "Instruction focused on blocking points"},
		{"cnt_meeting_020", "Meeting Listening: Start with the Risk", "listening", "intermediate", "meeting", "Let us start with the main risk first, and then we can discuss optional improvements if there is time.", "Meeting cue that prioritizes risk first"},
		{"cnt_meeting_021", "Meeting Speaking: State One Concern", "speaking_template", "intermediate", "meeting", "I have one concern. The wording is still heavy. Users may lose the main point too early.", "Short template for stating one concern"},
		{"cnt_meeting_022", "Meeting Speaking: Ask for a Clear Decision", "speaking_template", "intermediate", "meeting", "I want to be clear. Is today's decision to ship now or wait one more week?", "Decision clarification in short units"},
		{"cnt_rescue_015", "Rescue: Ask for Only the Main Change", "rescue", "intro", "rescue", "Can you tell me only the main change first?", "Rescue phrase for change-heavy explanations"},
		{"cnt_rescue_016", "Rescue: Ask for One Example", "rescue", "intro", "rescue", "Could you give me one example before the details?", "Rescue phrase for example-first support"},
		{"cnt_rescue_017", "Rescue: Ask to Split the Answer", "rescue", "intro", "rescue", "Could we split the answer into two short parts?", "Rescue phrase for splitting long answers"},
		{"cnt_rescue_018", "Rescue: Ask to Repeat the Last Part", "rescue", "intro", "rescue", "Please repeat only the last part once more.", "Rescue phrase for local repetition"},
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
