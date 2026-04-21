# Worker サービス

`services/worker` は、このアプリの知能部分を担う Python ワーカーです。
英語をそのまま大量に見せるのではなく、ワーキングメモリ負荷を下げるための分析結果を返します。

今の役割は大きく 2 つです。

- 入力テキストを、読みやすい単位や話しやすい単位に変換する
- その人の崩れやすさに合わせて、次に出す練習や支援を決める

## ディレクトリ構成

```text
services/worker
|- app/
|  |- analysis/
|  |- analytics_summary/
|  |- assessment/
|  |- collapse_patterns/
|  |- chunking/
|  |- http/
|  |- listening_plan/
|  |- practice_set/
|  |- reader_plan/
|  |- rescue_plan/
|  |- skeleton/
|  |- speaking_plan/
|  |- application.py
|  |- config.py
|  |- context_profile.py
|  |- models.py
|  |- observability.py
|  |- rate_limit.py
|  |- runtime.py
|  |- security.py
|  `- text_analysis.py
|- tests/
|- pyproject.toml
`- README.md
```

## 主な責務

- `app/chunking/`
  英文を意味チャンクに分割します。
- `app/skeleton/`
  主語・動詞・目的語に近い骨格を取り出します。
- `app/reader_plan/`
  読むときにどの chunk から見れば負荷が下がるかを返します。
- `app/listening_plan/`
  どこで止めると聞き取りやすいかを返します。
- `app/speaking_plan/`
  長文をそのまま保持せず、短文連結で話す手順を返します。
- `app/rescue_plan/`
  処理落ちしそうなときのレスキューフレーズを返します。
- `app/assessment/`
  初期診断用の負荷推定を返します。
- `app/collapse_patterns/`
  イベントログから崩れやすい地点を要約します。
- `app/analytics_summary/`
  診断結果と崩れ方をまとめて、次回のおすすめを返します。
- `app/practice_set/`
  1 本の英文から、読む・聞く・話す・レスキューの練習セットを生成します。

## 利用できる endpoint

- `GET /health`
- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- `POST /analyze/reader-plan`
- `POST /analyze/listening-plan`
- `POST /analyze/speaking-plan`
- `POST /analyze/rescue-plan`
- `POST /analyze/assessment`
- `POST /analyze/collapse-patterns`
- `POST /analyze/analytics-summary`
- `POST /analyze/practice-set`

## 返せる分析

- `reader-plan`
  読む順番、折りたたむ support、hotspot を返します。
- `listening-plan`
  pause point、推奨速度、聞くときの cue を返します。
- `speaking-plan`
  opener、bridge phrase、短文の speaking step を返します。
- `rescue-plan`
  overload 時の優先レスキューフレーズを返します。
- `assessment`
  読む・聞く・話すの負荷スコアと初期推奨モードを返します。
- `collapse-patterns`
  repeat、long pause、support open などから崩れやすい箇所を返します。
- `analytics-summary`
  診断結果と崩れ方をまとめ、次に何を優先すべきかを返します。
- `practice-set`
  1 本の英文から、読む・聞く・話す・レスキューの小さな練習タスク群を返します。

## `target_context`

`target_context` によってガイダンスと recommendation が少し変わります。

- `general`
- `self_intro`
- `research`
- `meeting`
- `daily`

例:

- `research` では `claim / method / result` を優先します。
- `meeting` では `decision / action item` を優先します。
- `self_intro` では `role / identity / goal` を優先します。

## セキュリティ

この worker はローカル分析サーバではなく、前段保護ありの内部サービスとして扱う前提です。

- API key 認証
- HMAC request signing
- request body サイズ制限
- text 長さ制限
- typed request validation
- request timeout
- rate limiting
- audit logging

## 起動

```bash
python -m app.server
```

## Docker ビルド

```bash
docker build -t mse-worker services/worker
```

## リクエスト例

```json
{
  "text": "In this study, we propose a memory safe interface.",
  "language": "en",
  "target_context": "research"
}
```

`assessment`、`analytics-summary`、`practice-set` では必要に応じて次も渡せます。

- `self_reported_difficulties`
- `fatigue_level`
- `session_events`

## テスト

```bash
python -m pytest tests -q
```
