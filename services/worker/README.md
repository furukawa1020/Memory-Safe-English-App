# Worker Service

`services/worker` is the Python analysis worker for chunking and skeleton extraction.

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

- `app/analysis/`: typed request models and operation dispatch
- `app/chunking/`: chunking service
- `app/skeleton/`: skeleton extraction service
- `app/http/request_parser.py`: HTTP request parsing and validation
- `app/http/routes.py`: endpoint to operation mapping
- `app/http/handlers.py`: transport and security guards
- `app/text_analysis.py`: shared heuristics used by chunking and skeleton extraction

## Current Features

- `GET /health`
- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- versioned analysis responses
- API key authentication
- HMAC request signing
- request body and text size limits
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

## Example Request

```json
{
  "text": "In this study, we propose a memory safe interface.",
  "language": "en"
}
```

The same request shape works for both analysis endpoints.

## Verify

```bash
python -m pytest tests -q
```
