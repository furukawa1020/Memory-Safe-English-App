# Memory Safe English

Memory Safe English is an English-learning product designed for users who struggle with working-memory load during reading, listening, and speaking.

This repository is a monorepo with four main parts:

- a Flutter mobile app
- a Go API
- a Python analysis worker
- a Rust proxy with short-lived cache management

## Repository Structure

```text
.
|- apps/
|  `- mobile/                 # Flutter app
|- services/
|  |- api/                    # Go REST API
|  |- worker/                 # Python analysis worker
|  `- proxy-rs/               # Rust proxy, cache, and GC
|- infra/
|  |- docker-compose.yml      # Local development stack
|  `- postgres/init/          # Initial SQL
|- docs/
|  |- architecture.md
|  |- api-outline.md
|  |- data-model.md
|  `- mvp-roadmap.md
`- scripts/
   |- bootstrap-mobile.ps1
   |- smoke-test.ps1
   `- start-dev-stack.ps1
```

## Services

- `services/api`: authentication, content delivery, sessions, and worker orchestration
- `services/worker`: chunk and skeleton analysis with request signing
- `services/proxy-rs`: reverse proxy, cache GC, admin endpoints, readiness checks
- `apps/mobile`: Flutter client for auth, content browsing, and reader flow

## Local Stack

The local stack is defined in [infra/docker-compose.yml](./infra/docker-compose.yml).

- `proxy`: `http://127.0.0.1:8070`
- `api`: `http://127.0.0.1:8080`
- `worker`: `http://127.0.0.1:8090`
- `postgres`: `127.0.0.1:5432`
- `redis`: `127.0.0.1:6379`

All services include health checks. The proxy uses `/ready`, so it becomes healthy only after the API and worker are ready.

## Recommended Local Workflow

### 1. Start the stack

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-dev-stack.ps1
```

This script:

- validates Docker availability
- runs `docker compose up -d --build`
- waits for all containers to become healthy
- runs the smoke test by default

### 2. Run the smoke test manually

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
```

The smoke test exercises:

- proxy readiness
- auth register or login
- content listing
- chunk analysis
- skeleton analysis
- proxy admin cache stats

### 3. Prepare the Flutter app

Flutter is not bundled in this repository. Install the Flutter SDK first, then run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1
```

The script creates missing platform scaffolding and runs `flutter pub get`.

### 4. Run the app in an Android emulator

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8070
```

`10.0.2.2` lets the Android emulator reach the host machine, and `8070` points the mobile app at the Rust proxy.

## Verification

Backend verification commands:

```bash
cd services/api && go test ./...
cd services/worker && python -m pytest tests -q
cd services/proxy-rs && cargo test
```

## Documentation

- [Architecture](./docs/architecture.md)
- [API Outline](./docs/api-outline.md)
- [Data Model](./docs/data-model.md)
- [MVP Roadmap](./docs/mvp-roadmap.md)
- [OpenAPI](./services/api/openapi/openapi.yaml)
