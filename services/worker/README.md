# Worker Service

`services/worker` は Python 製の分析 worker です。現在は `chunking` と `skeleton` を提供します。

## Structure

```text
services/worker
|- app/
|  |- analysis/
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

- `app/analysis/`: typed request schema と dispatch
- `app/chunking/`: chunking service
- `app/skeleton/`: skeleton extraction service
- `app/http/request_parser.py`: HTTP request から typed request への変換
- `app/http/routes.py`: endpoint と operation の対応
- `app/http/handlers.py`: transport と guards
- `app/text_analysis.py`: chunking / skeleton の共有ロジック

## Current Features

- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- versioned analysis responses
- API key auth
- HMAC request signing
- body / text size limits
- typed request validation
- request timeout
- rate limiting
- structured audit logging

## Run

```bash
python -m app.server
```

## Container

```bash
docker build -t mse-worker services/worker
```

## API

### `GET /health`

worker の疎通確認用 endpoint です。

### `POST /analyze/chunks`

```json
{
  "text": "In this study, we propose a memory safe interface.",
  "language": "en"
}
```

### `POST /analyze/skeleton`

request は `POST /analyze/chunks` と同じです。

## Verify

```bash
python -m pytest tests -q
```
