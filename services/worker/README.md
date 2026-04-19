# Worker Service

Python ベースの NLP / 解析ワーカーです。現在は `chunking` と `skeleton` を提供します。

## Structure

```text
services/worker
|- app/
|  |- chunking/
|  |- http/
|  |- skeleton/
|  |- application.py
|  |- config.py
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

## Design

- `app/analysis/`: request schema と analysis dispatch
- `app/chunking/`: chunk 分割ユースケース
- `app/skeleton/`: skeleton 抽出ユースケース
- `app/http/request_parser.py`: HTTP request から typed request への変換
- `app/http/routes.py`: endpoint と operation の対応
- `app/http/handlers.py`: HTTP transport と guard 適用
- `app/text_analysis.py`: chunking / skeleton の共有ロジック

## Current Features

- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- versioned analysis responses
- API key 認証
- HMAC request signing
- body / text size 制限
- typed request validation
- request timeout
- rate limiting
- structured audit logging

## Run

```bash
python -m app.server
```

## API

### `GET /health`

worker の疎通確認です。

### `POST /analyze/chunks`

request:

```json
{
  "text": "In this study, we propose a memory safe interface.",
  "language": "en"
}
```

### `POST /analyze/skeleton`

request は `POST /analyze/chunks` と同じです。
