# Worker Service

`services/worker` is the Python analysis worker for chunking, skeleton extraction, reader-plan generation, listening pause planning, speaking response planning, conversation rescue planning, onboarding assessment, collapse-pattern analysis, and analytics summary generation.

## Structure

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

## Design

- `app/analysis/`: typed request models and operation dispatch
- `app/chunking/`: chunking service
- `app/context_profile.py`: context-aware defaults for research, meeting, self-intro, and daily flows
- `app/assessment/`: onboarding profile estimation for initial mode recommendations
- `app/analytics_summary/`: next-step recommendations from assessment and collapse patterns
- `app/collapse_patterns/`: collapse-site analysis from session event traces
- `app/skeleton/`: skeleton extraction service
- `app/reader_plan/`: progressive reading plan generation for low-memory reading flows
- `app/listening_plan/`: pause-plan generation for lower-load listening passes
- `app/rescue_plan/`: rescue phrase planning for overload moments in conversation
- `app/speaking_plan/`: short-step response planning for low-load speaking support
- `app/http/request_parser.py`: HTTP request parsing and validation
- `app/http/routes.py`: endpoint to operation mapping
- `app/http/handlers.py`: transport and security guards
- `app/text_analysis.py`: shared heuristics used by chunking and skeleton extraction

## Current Features

- `GET /health`
- `POST /analyze/chunks`
- `POST /analyze/skeleton`
- `POST /analyze/reader-plan`
- `POST /analyze/listening-plan`
- `POST /analyze/rescue-plan`
- `POST /analyze/assessment`
- `POST /analyze/analytics-summary`
- `POST /analyze/collapse-patterns`
- `POST /analyze/speaking-plan`
- versioned analysis responses
- reader plans include focus steps, collapsed support chunks, overload hotspots, and display hints
- listening plans include pause checkpoints, replay cues, and recommended playback speed
- rescue plans include prioritized rescue phrases and a primary conversation strategy
- assessment returns initial reader/listening/speaking mode recommendations and display defaults
- analytics summary returns next focus recommendations from assessment and collapse signals
- collapse-pattern analysis turns session events into hotspot summaries and lighter-display recommendations
- speaking plans include short response steps, opener options, bridge phrases, and rescue phrases
- `target_context` changes cues and default guidance for research, self-intro, meeting, and daily use cases
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
  "language": "en",
  "target_context": "research"
}
```

The same request shape works for all analysis endpoints.

Supported `target_context` values:

- `general`
- `self_intro`
- `research`
- `meeting`
- `daily`

## Verify

```bash
python -m pytest tests -q
```
