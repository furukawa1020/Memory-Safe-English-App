# API Service

`services/api` is the Go REST API for authentication, content delivery, learning sessions, and worker orchestration.

## Structure

```text
services/api
|- cmd/server
|- internal/app
|- internal/config
|- internal/domain
|- internal/handlers
|- internal/httpjson
|- internal/httpx
|- internal/repository
|- internal/security
|- internal/service
|- internal/store/
|  |- memory/
|  `- postgres/
|- internal/workerclient
`- openapi
```

The implementation is separated as `repository -> service -> handler`.

- `repository`: persistence boundary
- `service`: use cases and business logic
- `handlers`: HTTP input/output
- `workerclient`: Python worker integration
- `app`: dependency wiring and server bootstrap

## Repository Backends

- default: in-memory store
- when `DATABASE_URL` is set: PostgreSQL store

The PostgreSQL implementation is split by responsibility.

- `postgres/users.go`: auth and user retrieval
- `postgres/refresh_tokens.go`: refresh token families and rotation state
- `postgres/sessions.go`: sessions and event logs
- `postgres/contents.go`: content CRUD
- `postgres/analysis_cache.go`: chunk and skeleton cache
- `postgres/scanners.go`: row-to-domain mapping

## Current Endpoints

- `GET /health`
- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/refresh`
- `GET /me`
- `POST /analysis/chunks`
- `POST /analysis/skeleton`
- `GET /contents`
- `POST /contents`
- `GET /contents/{id}`
- `PATCH /contents/{id}`
- `GET /contents/{id}/chunks`
- `GET /contents/{id}/skeleton`
- `POST /sessions/start`
- `POST /sessions/{id}/event`
- `POST /sessions/{id}/complete`

## Security Notes

- password hashing uses `PBKDF2-SHA256`
- access and refresh tokens use HMAC signatures
- refresh tokens rotate on every successful `/auth/refresh`
- replaying an older refresh token revokes the whole refresh-token family
- protected routes use Bearer token middleware
- auth routes use in-memory rate limiting for `login`, `register`, and `refresh`
- auth success, failure, and rate-limit decisions are written as audit logs
- expired refresh sessions are cleaned up on a background interval
- `X-Request-ID` is propagated to responses and logs
- `X-Content-Type-Options: nosniff` is set
- worker calls use API key + timestamp + HMAC signature

## Content Flow

`/contents/{id}/chunks` and `/contents/{id}/skeleton` analyze persisted content through the worker and cache the result in the repository layer. `PATCH /contents/{id}` invalidates related cached analysis.

## Run

```bash
go run ./cmd/server
```

Main environment variables:

- `API_HTTP_ADDR`: default `:8080`
- `APP_ENV`: default `development`
- `DATABASE_URL`: enables the PostgreSQL repository when set
- `AUTH_TOKEN_SECRET`: required in production
- `AUTH_ACCESS_TOKEN_TTL`: default `15m`
- `AUTH_REFRESH_TOKEN_TTL`: default `168h`
- `PASSWORD_HASH_ITERATIONS`: default `120000`
- `AUTH_RATE_LIMIT_WINDOW`: default `10m`
- `AUTH_RATE_LIMIT_LOGIN_MAX_ATTEMPTS`: default `10`
- `AUTH_RATE_LIMIT_REGISTER_MAX_ATTEMPTS`: default `5`
- `AUTH_RATE_LIMIT_REFRESH_MAX_ATTEMPTS`: default `20`
- `AUTH_REFRESH_CLEANUP_INTERVAL`: default `1h`
- `WORKER_BASE_URL`: default `http://127.0.0.1:8090`
- `WORKER_API_KEY`: worker API key
- `WORKER_SIGNATURE_KEY`: worker request signing key
- `WORKER_TIMEOUT`: default `5s`

## Container

```bash
docker build -t mse-api services/api
```

In compose, the API runs with `DATABASE_URL=postgres://mse:mse@postgres:5432/memory_safe_english?sslmode=disable`.

## Verify

```bash
go test ./...
```

OpenAPI lives at [openapi/openapi.yaml](./openapi/openapi.yaml).
