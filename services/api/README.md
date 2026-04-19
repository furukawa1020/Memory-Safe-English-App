# API Service

Go ベースの REST API サービスです。認証、ユーザー取得、学習セッション、content 管理、worker 連携による chunk 解析を担当します。

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

責務は `repository -> service -> handler` に分離しています。

- `repository`: 永続化の境界
- `service`: ユースケースと入力検証
- `handlers`: HTTP 入出力
- `workerclient`: Python worker との安全な通信
- `app`: DI とサーバ組み立て

この構成にしてあるので、in-memory 実装から PostgreSQL 実装へ差し替えるときも HTTP 層を大きく崩さず進められます。

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
- `POST /sessions/start`
- `POST /sessions/{id}/event`
- `POST /sessions/{id}/complete`

## Security Notes

- パスワードは `PBKDF2-SHA256` でハッシュ化
- access / refresh token は HMAC 署名
- protected route は Bearer token middleware で集中管理
- `X-Request-ID` をレスポンスへ付与
- `X-Content-Type-Options: nosniff` など基本ヘッダを付与
- worker 呼び出しは API key + timestamp + HMAC signature を使用

## Content Flow

`/contents/{id}/chunks` は content の本文を worker に渡して chunk 解析を取得します。結果は API 側で cache し、同じ content への再アクセスでは worker を再実行しません。`PATCH /contents/{id}` で本文が更新された場合は cache を無効化します。

## Run

```bash
go run ./cmd/server
```

主要な環境変数:

- `API_HTTP_ADDR`: default `:8080`
- `APP_ENV`: default `development`
- `AUTH_TOKEN_SECRET`: production では必須
- `AUTH_ACCESS_TOKEN_TTL`: default `15m`
- `AUTH_REFRESH_TOKEN_TTL`: default `168h`
- `PASSWORD_HASH_ITERATIONS`: default `120000`
- `WORKER_BASE_URL`: default `http://127.0.0.1:8090`
- `WORKER_API_KEY`: worker API key
- `WORKER_SIGNATURE_KEY`: worker request signing key
- `WORKER_TIMEOUT`: default `5s`

## Verify

```bash
go test ./...
```

OpenAPI 契約は [openapi/openapi.yaml](./openapi/openapi.yaml) にあります。
