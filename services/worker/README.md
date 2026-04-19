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

## Current Features

- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- API key 認証
- HMAC request signing
- body / text size 制限
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
