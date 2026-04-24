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
    )
ON CONFLICT (id) DO NOTHING;
