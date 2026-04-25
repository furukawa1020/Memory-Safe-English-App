INSERT INTO contents (
    id,
    title,
    content_type,
    level,
    topic,
    language,
    raw_text,
    summary_text
)
VALUES
    ('cnt_self_intro_029', 'Self Introduction: Presentation Goal', 'reading', 'intro', 'self_intro', 'en', 'When I present in English, my goal is to stay clear and avoid losing the main sentence halfway through.', 'Presentation goal in one sentence'),
    ('cnt_self_intro_030', 'Self Introduction: Short Closing', 'speaking_template', 'intro', 'self_intro', 'en', 'That is my short introduction. Thank you for listening.', 'Short closing for introductions'),
    ('cnt_daily_045', 'Daily Reading: Ask for the Main Entrance', 'reading', 'intro', 'daily', 'en', 'Could you tell me which entrance is best if I only need the information desk?', 'Main-entrance question'),
    ('cnt_daily_046', 'Daily Listening: Short Safety Notice', 'listening', 'intro', 'daily', 'en', 'Please keep your bag with you and move away from the door after boarding.', 'Safety notice with two short actions'),
    ('cnt_daily_047', 'Daily Speaking: Ask for the Topic First', 'speaking_template', 'intro', 'daily', 'en', 'Before the details, could you tell me the topic first?', 'Topic-first speaking template'),
    ('cnt_research_047', 'Research Reading: Why UI Matters', 'reading', 'intermediate', 'research', 'en', 'Interface design matters because overload can appear even when the learner already knows the vocabulary.', 'Why UI matters in one sentence'),
    ('cnt_research_048', 'Research Listening: Main Limitation', 'listening', 'intermediate', 'research', 'en', 'The main limitation is that we still need evidence from live conversation, not only controlled reading tasks.', 'Main limitation for listening practice'),
    ('cnt_research_049', 'Research Speaking: Core Takeaway', 'speaking_template', 'intermediate', 'research', 'en', 'The core takeaway is simple. Lower holding demand helps understanding.', 'Core takeaway in short units'),
    ('cnt_research_050', 'Research Reading: User Benefit', 'reading', 'intermediate', 'research', 'en', 'Users benefit when the system protects the core relation before extra detail starts to compete for attention.', 'User benefit statement'),
    ('cnt_meeting_041', 'Meeting Reading: Ask for One Priority', 'reading', 'intermediate', 'meeting', 'en', 'If we choose only one priority for this week, which priority should it be?', 'One-priority meeting question'),
    ('cnt_meeting_042', 'Meeting Listening: Confirm the Scope', 'listening', 'intermediate', 'meeting', 'en', 'To confirm the scope, today we will review only the first two screens and not the full onboarding flow.', 'Scope confirmation for meetings'),
    ('cnt_meeting_043', 'Meeting Speaking: Short Progress Update', 'speaking_template', 'intermediate', 'meeting', 'en', 'Progress is steady. The main flow works. The final check is still open.', 'Short progress update'),
    ('cnt_meeting_044', 'Meeting Reading: Ask for One Example', 'reading', 'intermediate', 'meeting', 'en', 'Could you give one example of the user problem before we discuss the full report?', 'Example-first meeting question'),
    ('cnt_rescue_043', 'Rescue: Ask for the Short Version First', 'rescue', 'intro', 'rescue', 'en', 'Could you give me the short version first?', 'Rescue phrase for short-version-first'),
    ('cnt_rescue_044', 'Rescue: Ask for the Core Idea Again', 'rescue', 'intro', 'rescue', 'en', 'Could you say the core idea again?', 'Rescue phrase for core idea repetition'),
    ('cnt_rescue_045', 'Rescue: Ask for the Main Task', 'rescue', 'intro', 'rescue', 'en', 'What is the main task for me?', 'Rescue phrase for main task'),
    ('cnt_rescue_046', 'Rescue: Ask to Hold the Last Sentence', 'rescue', 'intro', 'rescue', 'en', 'Can we keep the last sentence visible for a moment?', 'Rescue phrase for visual sentence hold'),
    ('cnt_rescue_047', 'Rescue: Ask for One More Pause', 'rescue', 'intro', 'rescue', 'en', 'Could you pause once more before the next point?', 'Rescue phrase for extra pause'),
    ('cnt_rescue_048', 'Rescue: Ask for a Simpler Explanation', 'rescue', 'intro', 'rescue', 'en', 'Could you explain it in a simpler way first?', 'Rescue phrase for simpler explanation'),
    ('cnt_rescue_049', 'Rescue: Ask for the First Step Again', 'rescue', 'intro', 'rescue', 'en', 'Could you repeat the first step one more time?', 'Rescue phrase for first-step repetition')
ON CONFLICT (id) DO NOTHING;
