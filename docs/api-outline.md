# API Outline

## Principles

- REST API ベース
- JSON を基本とする
- 認証は token ベース
- 契約は OpenAPI で管理
- イベントログ API は高頻度書き込みを前提に簡潔に保つ

## Core Resources

- `auth`
- `me`
- `settings`
- `contents`
- `sessions`
- `analytics`
- `speech`
- `admin`

## Auth

### `POST /auth/register`

用途:

- メールアドレスで新規登録

想定 request:

```json
{
  "email": "user@example.com",
  "password": "********",
  "display_name": "Aki",
  "agreed_to_terms": true
}
```

想定 response:

```json
{
  "user_id": "usr_123",
  "access_token": "jwt-or-session-token",
  "refresh_token": "refresh-token"
}
```

### `POST /auth/login`

用途:

- メールログイン

### `POST /auth/oauth`

用途:

- Google / Apple ログイン

### `POST /auth/refresh`

用途:

- アクセストークン更新

### `POST /auth/logout`

用途:

- セッション無効化

## User

### `GET /me`

用途:

- ログイン中ユーザー取得

### `PATCH /me`

用途:

- プロファイル更新

### `GET /me/settings`

用途:

- UI / 音声 / 学習設定取得

### `PATCH /me/settings`

用途:

- 負荷調整設定更新

推奨設定項目:

- `chunk_length`
- `font_scale`
- `line_spacing`
- `highlight_style`
- `audio_speed`
- `pause_frequency`
- `show_japanese_support`
- `simple_ui_mode`

## Onboarding

### `POST /onboarding/assessment/start`

用途:

- 初期診断セッション開始

### `POST /onboarding/assessment/submit`

用途:

- 回答送信と評価

### `GET /onboarding/result`

用途:

- 推定プロファイル返却

想定 response:

```json
{
  "profile": {
    "reading_load": "high",
    "listening_load": "medium",
    "speaking_load": "high"
  },
  "recommended_mode": "assisted",
  "initial_settings": {
    "chunk_length": "short",
    "pause_frequency": "high",
    "show_japanese_support": true
  }
}
```

## Content

### `GET /contents`

用途:

- コンテンツ一覧取得

フィルタ:

- `type`
- `level`
- `topic`
- `goal`

### `GET /contents/{id}`

用途:

- コンテンツ詳細取得

### `GET /contents/{id}/chunks`

用途:

- チャンク済み文と骨格情報取得

想定 response:

```json
{
  "content_id": "cnt_123",
  "title": "Research Introduction",
  "display_modes": ["normal", "chunk", "skeleton", "assisted"],
  "chunks": [
    {
      "order": 1,
      "text": "In this study,",
      "role": "modifier"
    },
    {
      "order": 2,
      "text": "we propose",
      "role": "core"
    },
    {
      "order": 3,
      "text": "a memory-safe interface",
      "role": "object"
    }
  ]
}
```

### `GET /contents/recommended`

用途:

- 最近の処理落ち傾向に応じた推奨教材取得

## Sessions

### `POST /sessions/start`

用途:

- 学習セッション開始

想定 request:

```json
{
  "mode": "reading",
  "content_id": "cnt_123"
}
```

### `POST /sessions/{id}/event`

用途:

- 学習イベント送信

想定 request:

```json
{
  "event_type": "chunk_focus_changed",
  "occurred_at": "2026-04-15T09:00:00Z",
  "payload": {
    "chunk_order": 2,
    "duration_ms": 1840
  }
}
```

### `POST /sessions/{id}/complete`

用途:

- セッション終了

## Analytics

### `GET /analytics/summary`

用途:

- 今日 / 週次の簡易指標返却

### `GET /analytics/collapse-patterns`

用途:

- どこで崩れやすいかのパターン返却

### `GET /analytics/recommendations`

用途:

- 次回学習推薦返却

## Speech

### `POST /speech/upload`

用途:

- 録音メタデータ登録

### `POST /speech/analyze`

用途:

- 発話解析依頼

## Admin

### `POST /admin/contents`

用途:

- コンテンツ登録

### `PATCH /admin/contents/{id}`

用途:

- コンテンツ更新

## Event Taxonomy For MVP

最低限、以下のイベント種別は初期から固定で持ちます。

- `session_started`
- `chunk_focus_changed`
- `chunk_replayed`
- `skeleton_mode_opened`
- `translation_opened`
- `audio_played`
- `audio_paused`
- `audio_replayed`
- `speech_recording_started`
- `speech_recording_completed`
- `self_rating_submitted`
- `session_completed`
