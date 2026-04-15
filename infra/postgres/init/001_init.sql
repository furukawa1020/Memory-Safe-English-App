CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE,
    password_hash TEXT,
    auth_provider TEXT NOT NULL,
    subscription_status TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
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
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
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

CREATE TABLE IF NOT EXISTS contents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    chunk_order INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    chunk_role TEXT,
    syntactic_label TEXT,
    skeleton_rank INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (content_id, chunk_order)
);

CREATE TABLE IF NOT EXISTS audio_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID REFERENCES contents(id) ON DELETE SET NULL,
    storage_key TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    transcript_text TEXT,
    segmentation_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    mode TEXT NOT NULL,
    content_id UUID REFERENCES contents(id) ON DELETE SET NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    completion_state TEXT NOT NULL DEFAULT 'started'
);

CREATE TABLE IF NOT EXISTS exercise_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL,
    mode TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    self_rating INTEGER,
    result_summary JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS event_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_logs_user_id ON event_logs (user_id);
CREATE INDEX IF NOT EXISTS idx_event_logs_session_id ON event_logs (session_id);
CREATE INDEX IF NOT EXISTS idx_event_logs_event_type ON event_logs (event_type);
CREATE INDEX IF NOT EXISTS idx_event_logs_created_at ON event_logs (created_at);

CREATE TABLE IF NOT EXISTS analytics_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL,
    reading_load_score NUMERIC(5,2),
    listening_load_score NUMERIC(5,2),
    speaking_load_score NUMERIC(5,2),
    collapse_pattern_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, snapshot_date)
);

CREATE TABLE IF NOT EXISTS rescue_phrases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category TEXT NOT NULL,
    phrase_en TEXT NOT NULL,
    phrase_ja TEXT,
    audio_asset_id UUID REFERENCES audio_assets(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO rescue_phrases (category, phrase_en, phrase_ja)
VALUES
    ('slow_down', 'Please say that more slowly.', 'もう少しゆっくり話してください。'),
    ('ask_repeat', 'One more time, please.', 'もう一度お願いします。'),
    ('clarify', 'Do you mean ...?', '...という意味ですか。'),
    ('ask_summary', 'Can you say it in a shorter way?', 'もっと短く言ってもらえますか。'),
    ('buy_time', 'Let me think for a moment.', '少し考えさせてください。')
ON CONFLICT DO NOTHING;
