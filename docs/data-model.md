# Data Model

## Design Notes

- PII と学習ログは分離する
- 初期は厳密な正規化よりも、解析しやすいイベント構造を優先する
- 将来的な多言語展開を見据えて `language` を明示する
- 処理落ち分析のため、結果より過程のログを重視する

## Core Tables

### `users`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| email | text | unique, nullable for oauth-only users |
| password_hash | text | nullable for oauth users |
| auth_provider | text | email, google, apple |
| subscription_status | text | free, trial, active, canceled |
| created_at | timestamptz | not null |
| updated_at | timestamptz | not null |

### `user_profiles`

| column | type | notes |
| --- | --- | --- |
| user_id | uuid | PK, FK to users |
| display_name | text | not null |
| native_language | text | default `ja` |
| learning_goal | text | nullable |
| target_context | text | daily, research, work |
| self_reported_difficulty | jsonb | difficulty questionnaire summary |
| onboarding_completed | boolean | default false |
| created_at | timestamptz | not null |
| updated_at | timestamptz | not null |

### `user_settings`

| column | type | notes |
| --- | --- | --- |
| user_id | uuid | PK, FK to users |
| chunk_length | text | short, medium, long |
| font_scale | numeric(4,2) | default 1.0 |
| line_spacing | numeric(4,2) | default 1.4 |
| color_theme | text | accessible palette id |
| highlight_style | text | underline, tint, outline |
| audio_speed | numeric(4,2) | default 1.0 |
| pause_frequency | text | low, medium, high |
| show_japanese_support | boolean | default true |
| simple_ui_mode | boolean | default false |
| created_at | timestamptz | not null |
| updated_at | timestamptz | not null |

### `contents`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| title | text | not null |
| content_type | text | reading, listening, speaking_template, rescue |
| level | text | intro, basic, intermediate, advanced |
| topic | text | self_intro, research, daily |
| language | text | default `en` |
| raw_text | text | source text |
| summary_text | text | optional |
| created_at | timestamptz | not null |
| updated_at | timestamptz | not null |

### `content_chunks`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| content_id | uuid | FK to contents |
| chunk_order | integer | not null |
| chunk_text | text | not null |
| chunk_role | text | core, subject, verb, object, modifier |
| syntactic_label | text | parser output |
| skeleton_rank | integer | lower means more central |
| created_at | timestamptz | not null |

### `audio_assets`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| content_id | uuid | FK to contents |
| storage_key | text | not null |
| duration_ms | integer | not null |
| transcript_text | text | optional |
| segmentation_json | jsonb | pause points and segment metadata |
| created_at | timestamptz | not null |

### `sessions`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| user_id | uuid | FK to users |
| mode | text | reading, listening, speaking, onboarding |
| content_id | uuid | nullable FK |
| started_at | timestamptz | not null |
| completed_at | timestamptz | nullable |
| completion_state | text | started, completed, abandoned |

### `exercise_attempts`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| user_id | uuid | FK to users |
| exercise_id | uuid | app-defined logical id |
| mode | text | reading, listening, speaking |
| started_at | timestamptz | not null |
| completed_at | timestamptz | nullable |
| self_rating | integer | 1-5 |
| result_summary | jsonb | summarized outcome |

### `event_logs`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| user_id | uuid | FK to users |
| session_id | uuid | FK to sessions |
| event_type | text | indexed |
| payload_json | jsonb | event-specific payload |
| created_at | timestamptz | indexed |

### `analytics_snapshots`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| user_id | uuid | FK to users |
| snapshot_date | date | not null |
| reading_load_score | numeric(5,2) | nullable |
| listening_load_score | numeric(5,2) | nullable |
| speaking_load_score | numeric(5,2) | nullable |
| collapse_pattern_summary | jsonb | ranked pain points |
| created_at | timestamptz | not null |

### `rescue_phrases`

| column | type | notes |
| --- | --- | --- |
| id | uuid | PK |
| category | text | ask_repeat, ask_summary, slow_down, clarify, buy_time |
| phrase_en | text | not null |
| phrase_ja | text | optional |
| audio_asset_id | uuid | nullable FK |
| is_active | boolean | default true |
| created_at | timestamptz | not null |

## Event Design Guidance

`event_logs.payload_json` には、機能別に次を入れます。

### Reading events

- `content_id`
- `chunk_order`
- `display_mode`
- `duration_ms`
- `opened_translation`

### Listening events

- `audio_asset_id`
- `segment_index`
- `speed`
- `replay_count`
- `revealed_transcript`

### Speaking events

- `template_id`
- `utterance_length_tokens`
- `silence_ms`
- `retry_count`
- `transcript_excerpt`

## Derived Metrics

イベントから次を後段で算出します。

- 崩れやすいチャンク長
- 修飾成分での停止率
- 音声長ごとの離脱率
- 骨格表示依存度
- 日本語補助依存度
- 話すときの無音増加傾向
