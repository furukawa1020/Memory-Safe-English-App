# API Service

`services/api` は Go 製の REST API です。認証、コンテンツ配信、学習セッション、worker 連携を担当します。

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
|- internal/store/memory
|- internal/workerclient
`- openapi
```

実装は `repository -> service -> handler` で分離しています。

- `repository`: 永続化境界
- `service`: ユースケースと業務ロジック
- `handlers`: HTTP 入出力
- `workerclient`: Python worker 連携
- `app`: DI とサーバ初期化

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

- password hash は `PBKDF2-SHA256`
- access / refresh token は HMAC 署名
- protected route は Bearer token middleware で保護
- `X-Request-ID` をレスポンスとログに反映
- `X-Content-Type-Options: nosniff` を付与
- worker 呼び出しは API key + timestamp + HMAC signature

## Content Flow

`/contents/{id}/chunks` と `/contents/{id}/skeleton` は保存済み content を worker で解析し、API 側で短期 cache します。`PATCH /contents/{id}` で本文が更新された場合は関連 cache を無効化します。

## Run

```bash
go run ./cmd/server
```

主な環境変数:

- `API_HTTP_ADDR`: default `:8080`
- `APP_ENV`: default `development`
- `DATABASE_URL`: set すると PostgreSQL repository を利用
- `AUTH_TOKEN_SECRET`: production では必須
- `AUTH_ACCESS_TOKEN_TTL`: default `15m`
- `AUTH_REFRESH_TOKEN_TTL`: default `168h`
- `PASSWORD_HASH_ITERATIONS`: default `120000`
- `WORKER_BASE_URL`: default `http://127.0.0.1:8090`
- `WORKER_API_KEY`: worker API key
- `WORKER_SIGNATURE_KEY`: worker request signing key
- `WORKER_TIMEOUT`: default `5s`

## Container

```bash
docker build -t mse-api services/api
```

compose 経由では `DATABASE_URL=postgres://mse:mse@postgres:5432/memory_safe_english?sslmode=disable` を使って PostgreSQL backend で起動します。

## Verify

```bash
go test ./...
```

OpenAPI は [openapi/openapi.yaml](./openapi/openapi.yaml) にあります。
