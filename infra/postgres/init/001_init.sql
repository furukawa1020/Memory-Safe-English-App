CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE,
    password_hash TEXT,
    auth_provider TEXT NOT NULL,
    subscription_status TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_profiles (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL,
    native_language TEXT NOT NULL DEFAULT 'ja',
    learning_goal TEXT,
    target_context TEXT,
    self_reported_difficulty JSONB NOT NULL DEFAULT '{}'::jsonb,
    onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_settings (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    chunk_length TEXT NOT NULL DEFAULT 'short',
    font_scale NUMERIC(4,2) NOT NULL DEFAULT 1.0,
    line_spacing NUMERIC(4,2) NOT NULL DEFAULT 1.4,
    color_theme TEXT NOT NULL DEFAULT 'calm-default',
    highlight_style TEXT NOT NULL DEFAULT 'tint',
    audio_speed NUMERIC(4,2) NOT NULL DEFAULT 1.0,
    pause_frequency TEXT NOT NULL DEFAULT 'high',
    show_japanese_support BOOLEAN NOT NULL DEFAULT TRUE,
    simple_ui_mode BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS refresh_token_families (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS refresh_sessions (
    id TEXT PRIMARY KEY,
    family_id TEXT NOT NULL REFERENCES refresh_token_families(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    replaced_by_token_id TEXT REFERENCES refresh_sessions(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_refresh_sessions_family_id ON refresh_sessions (family_id);
CREATE INDEX IF NOT EXISTS idx_refresh_sessions_user_id ON refresh_sessions (user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_sessions_expires_at ON refresh_sessions (expires_at);

CREATE TABLE IF NOT EXISTS contents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content_type TEXT NOT NULL,
    level TEXT NOT NULL,
    topic TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'en',
    raw_text TEXT NOT NULL,
    summary_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS content_chunks (
    id TEXT PRIMARY KEY,
    content_id TEXT NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    chunk_order INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    chunk_role TEXT,
    syntactic_label TEXT,
    skeleton_rank INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (content_id, chunk_order)
);

CREATE TABLE IF NOT EXISTS audio_assets (
    id TEXT PRIMARY KEY,
    content_id TEXT REFERENCES contents(id) ON DELETE SET NULL,
    storage_key TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    transcript_text TEXT,
    segmentation_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode TEXT NOT NULL,
    content_id TEXT REFERENCES contents(id) ON DELETE SET NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    completion_state TEXT NOT NULL DEFAULT 'started'
);

CREATE TABLE IF NOT EXISTS exercise_attempts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    exercise_id TEXT NOT NULL,
    mode TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    self_rating INTEGER,
    result_summary JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS event_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_logs_user_id ON event_logs (user_id);
CREATE INDEX IF NOT EXISTS idx_event_logs_session_id ON event_logs (session_id);
CREATE INDEX IF NOT EXISTS idx_event_logs_event_type ON event_logs (event_type);
CREATE INDEX IF NOT EXISTS idx_event_logs_created_at ON event_logs (created_at);

CREATE TABLE IF NOT EXISTS analytics_snapshots (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL,
    reading_load_score NUMERIC(5,2),
    listening_load_score NUMERIC(5,2),
    speaking_load_score NUMERIC(5,2),
    collapse_pattern_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, snapshot_date)
);

CREATE TABLE IF NOT EXISTS rescue_phrases (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::text,
    category TEXT NOT NULL,
    phrase_en TEXT NOT NULL,
    phrase_ja TEXT,
    audio_asset_id TEXT REFERENCES audio_assets(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS content_analysis_cache (
    content_id TEXT PRIMARY KEY REFERENCES contents(id) ON DELETE CASCADE,
    chunking_result JSONB,
    skeleton_result JSONB,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO rescue_phrases (category, phrase_en, phrase_ja)
VALUES
    ('slow_down', 'Please say that more slowly.', 'もう少しゆっくり話してください。'),
    ('ask_repeat', 'One more time, please.', 'もう一度お願いします。'),
    ('clarify', 'Do you mean ...?', '...という意味ですか。'),
    ('ask_summary', 'Can you say it in a shorter way?', 'もっと短く言ってもらえますか。'),
    ('buy_time', 'Let me think for a moment.', '少し考えさせてください。')
ON CONFLICT DO NOTHING;

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
        'cnt_self_intro_001',
        'Self Introduction',
        'reading',
        'intro',
        'self_intro',
        'en',
        'Hello, my name is Aki, and I study human computer interaction at university.',
        'Simple self introduction'
    ),
    (
        'cnt_research_001',
        'Research Presentation Opening',
        'reading',
        'intermediate',
        'research',
        'en',
        'In this study, we propose a memory safe interface that reduces cognitive overload during English reading.',
        'Research opening sentence'
    )
ON CONFLICT (id) DO NOTHING;
