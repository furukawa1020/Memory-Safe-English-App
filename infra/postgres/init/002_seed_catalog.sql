UPDATE rescue_phrases
SET phrase_ja = CASE phrase_en
    WHEN 'Please say that more slowly.' THEN 'もう少しゆっくり話してください。'
    WHEN 'One more time, please.' THEN 'もう一度お願いします。'
    WHEN 'Do you mean ...?' THEN '...という意味ですか。'
    WHEN 'Can you say it in a shorter way?' THEN 'もっと短く言ってもらえますか。'
    WHEN 'Let me think for a moment.' THEN '少し考えさせてください。'
    ELSE phrase_ja
END
WHERE phrase_en IN (
    'Please say that more slowly.',
    'One more time, please.',
    'Do you mean ...?',
    'Can you say it in a shorter way?',
    'Let me think for a moment.'
);

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
    (
        'cnt_self_intro_002',
        'Short Self Introduction Template',
        'speaking_template',
        'intro',
        'self_intro',
        'en',
        'Hello. My name is Hana. I study psychology. Today I want to talk about learning support.',
        'Short self introduction in stable short units'
    ),
    (
        'cnt_self_intro_003',
        'Self Introduction: Research Interest',
        'reading',
        'intro',
        'self_intro',
        'en',
        'I am interested in how people understand difficult information when they feel tired or overloaded.',
        'Interest statement with one clear idea'
    ),
    (
        'cnt_self_intro_004',
        'Self Introduction: Goal',
        'speaking_template',
        'intro',
        'self_intro',
        'en',
        'My goal is simple. I want to explain my work clearly. I do not need long sentences.',
        'Goal-focused self introduction in short units'
    ),
    (
        'cnt_daily_001',
        'Ordering Coffee',
        'reading',
        'intro',
        'daily',
        'en',
        'I would like a small latte, please. If possible, can I get it with oat milk?',
        'Short daily request at a cafe'
    ),
    (
        'cnt_daily_002',
        'Daily Listening: Delayed Train',
        'listening',
        'intro',
        'daily',
        'en',
        'The train is delayed by ten minutes. Please wait on platform three for the next announcement.',
        'Simple delay announcement with one action point'
    ),
    (
        'cnt_daily_003',
        'Asking for Directions',
        'reading',
        'intro',
        'daily',
        'en',
        'Excuse me, is this the right way to the library, or should I turn left at the next corner?',
        'Direction question with one contrast'
    ),
    (
        'cnt_daily_004',
        'Daily Listening: Payment Counter',
        'listening',
        'intro',
        'daily',
        'en',
        'Please place your basket here, and pay at counter two when the light turns green.',
        'Short instruction with two steps'
    ),
    (
        'cnt_daily_005',
        'Daily Speaking: Simple Request',
        'speaking_template',
        'intro',
        'daily',
        'en',
        'I need help. This part is difficult. Could you explain the first step?',
        'Simple request for help using short units'
    ),
    (
        'cnt_daily_006',
        'Daily Reading: Appointment Change',
        'reading',
        'intro',
        'daily',
        'en',
        'I am sorry, but I need to move our appointment to tomorrow afternoon because I have a class this morning.',
        'Appointment change with one reason'
    ),
    (
        'cnt_research_002',
        'Research Method Overview',
        'reading',
        'intermediate',
        'research',
        'en',
        'We compared the new interface with a standard reader and measured how often participants lost the main point of the sentence.',
        'Method sentence with comparison and measurement'
    ),
    (
        'cnt_research_003',
        'Research Result Listening',
        'listening',
        'intermediate',
        'research',
        'en',
        'Participants using the new interface reported lower overload and gave more accurate summaries after each reading task.',
        'Result sentence with two outcomes'
    ),
    (
        'cnt_research_004',
        'Research Explanation Template',
        'speaking_template',
        'intermediate',
        'research',
        'en',
        'Our topic is reading overload. We built a safer interface. The main result is lower cognitive strain.',
        'Three-step research explanation template'
    ),
    (
        'cnt_research_005',
        'Research Motivation',
        'reading',
        'intermediate',
        'research',
        'en',
        'Many learners know the vocabulary, but they still lose the sentence when too many details arrive at the same time.',
        'Problem statement for overload-sensitive learners'
    ),
    (
        'cnt_research_006',
        'Research Listening: Limitation',
        'listening',
        'intermediate',
        'research',
        'en',
        'One limitation is that we tested short passages first, so we still need to evaluate longer academic texts.',
        'Limitation with one future task'
    ),
    (
        'cnt_research_007',
        'Research Speaking: Main Claim',
        'speaking_template',
        'intermediate',
        'research',
        'en',
        'The main claim is clear. Lower memory load helps comprehension. Our interface supports that process.',
        'Short speaking template for claim delivery'
    ),
    (
        'cnt_research_008',
        'Research Reading: Future Work',
        'reading',
        'intermediate',
        'research',
        'en',
        'In future work, we plan to personalize pause timing and display density for each learner profile.',
        'Future work sentence with two targets'
    ),
    (
        'cnt_meeting_001',
        'Meeting Decision Summary',
        'reading',
        'intermediate',
        'meeting',
        'en',
        'We decided to move the user test to Friday, and Ken will send the updated schedule by this afternoon.',
        'Decision plus action item'
    ),
    (
        'cnt_meeting_002',
        'Meeting Listening: Next Action',
        'listening',
        'intermediate',
        'meeting',
        'en',
        'Before the next meeting, please review the draft slides and write down one risk we should discuss.',
        'Short meeting instruction with one task'
    ),
    (
        'cnt_meeting_003',
        'Meeting Reading: Clarify Priority',
        'reading',
        'intermediate',
        'meeting',
        'en',
        'Our first priority is stability, and we can discuss visual improvements after the release schedule is fixed.',
        'Priority sentence with one deferment'
    ),
    (
        'cnt_meeting_004',
        'Meeting Listening: Schedule Change',
        'listening',
        'intermediate',
        'meeting',
        'en',
        'The client moved the review to next Tuesday, so please send your comments by Monday evening.',
        'Schedule update with one deadline'
    ),
    (
        'cnt_meeting_005',
        'Meeting Speaking: Report Status',
        'speaking_template',
        'intermediate',
        'meeting',
        'en',
        'The status is simple. The core feature works. The remaining risk is test coverage.',
        'Status update in three short units'
    ),
    (
        'cnt_meeting_006',
        'Meeting Reading: Action Owner',
        'reading',
        'intermediate',
        'meeting',
        'en',
        'Mika will prepare the notes, and I will share the summary with the group after lunch.',
        'Action ownership sentence with two people'
    ),
    (
        'cnt_rescue_001',
        'Rescue: Ask for the Main Point',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Can you tell me the main point first?',
        'Rescue phrase for overload during explanations'
    ),
    (
        'cnt_rescue_002',
        'Rescue: Ask for a Shorter Version',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Could you say that in a shorter way?',
        'Rescue phrase to reduce sentence length'
    ),
    (
        'cnt_rescue_003',
        'Rescue: Ask to Repeat Slowly',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Could you repeat that more slowly?',
        'Rescue phrase for fast speech'
    ),
    (
        'cnt_rescue_004',
        'Rescue: Buy Time',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Please give me a moment to think.',
        'Rescue phrase to create a short pause'
    ),
    (
        'cnt_rescue_005',
        'Rescue: Confirm Meaning',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Do you mean the deadline is tomorrow?',
        'Rescue phrase to confirm one key meaning'
    ),
    (
        'cnt_rescue_006',
        'Rescue: Ask for One Step',
        'rescue',
        'intro',
        'rescue',
        'en',
        'Can you show me the first step only?',
        'Rescue phrase to reduce multi-step overload'
    )
ON CONFLICT (id) DO NOTHING;
