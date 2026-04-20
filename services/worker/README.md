# Worker サービス

`services/worker` は、ワーキングメモリ負荷を下げるための分析ロジックを担う Python ワーカーです。  
英語をそのまま処理させるのではなく、`読む` `聞く` `話す` `レスキュー` `初期診断` `処理落ち分析` を、低負荷な学習 UI につなぎやすい形へ変換します。

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

## 役割

- `app/analysis/`
  分析リクエストの入力モデルと operation 振り分け
- `app/chunking/`
  英文を意味チャンクへ分割
- `app/skeleton/`
  文の骨格抽出
- `app/reader_plan/`
  読むときに何を先に見せるべきかを返す
- `app/listening_plan/`
  どこで止めると保持負荷が下がるかを返す
- `app/speaking_plan/`
  長文ではなく短文連結で話す手順を返す
- `app/rescue_plan/`
  会話中に詰まったときの定型レスキュー表現を返す
- `app/assessment/`
  初期診断として、表示密度や推奨モードの初期値を返す
- `app/collapse_patterns/`
  イベントログから処理落ち箇所と崩れ方を要約する
- `app/analytics_summary/`
  診断結果と処理落ち傾向をまとめて次回の推奨練習を返す
- `app/context_profile.py`
  `research` `meeting` `self_intro` `daily` など場面別の既定動作
- `app/http/`
  HTTP 受付、バリデーション、認証、署名検証

## 現在の主な機能

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

## 返せる内容

- `reader-plan`
  主軸チャンク、補助情報の折りたたみ候補、表示ヒント、負荷 hotspot
- `listening-plan`
  停止点、再確認キュー、推奨再生速度
- `speaking-plan`
  短文ステップ、opener、bridge phrase、レスキュー文
- `rescue-plan`
  優先度つきの聞き返し・要約依頼・時間稼ぎフレーズ
- `assessment`
  読む・聞く・話すの負荷スコア、初期推奨モード、表示密度
- `collapse-patterns`
  崩れやすい chunk、理由、推奨表示、`reading/listening/speaking` のどれ寄りか
- `analytics-summary`
  assessment と collapse-patterns をまとめた次回推奨

## 文脈別の出し分け

`target_context` によってガイダンスや recommendation が変わります。

- `general`
- `self_intro`
- `research`
- `meeting`
- `daily`

たとえば、

- `research` では `claim / method / result` を優先
- `meeting` では `decision / action item` を優先
- `self_intro` では `role / identity / goal` を優先

するように設計しています。

## セキュリティ

- API key 認証
- HMAC request signing
- request body サイズ制限
- text 長制限
- typed request validation
- request timeout
- rate limiting
- audit logging

## 実行

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

`assessment` や `analytics-summary` では、必要に応じて以下も渡せます。

- `self_reported_difficulties`
- `fatigue_level`
- `session_events`

## 検証

```bash
python -m pytest tests -q
```
