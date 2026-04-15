# API Service

Go 製 REST API サービスの配置先です。

初期責務:

- 認証認可
- ユーザー / 設定
- コンテンツ配信
- セッション管理
- イベントログ受付
- 分析結果返却

推奨ディレクトリ案:

```text
services/api
├─ cmd/server
├─ internal/auth
├─ internal/users
├─ internal/contents
├─ internal/sessions
├─ internal/events
├─ internal/analytics
└─ openapi
```

## Current Bootstrap

現時点では、標準ライブラリだけで起動できる最小 API 基盤を入れています。

含まれるエンドポイント:

- `GET /health`
- `POST /auth/register`
- `POST /auth/login`
- `GET /me`
- `POST /sessions/start`
- `POST /sessions/{id}/event`
- `POST /sessions/{id}/complete`

認証はまだ開発用の簡易フローです。`/auth/register` で返る `user.user_id` を `X-User-ID` ヘッダに入れると `/me` や `/sessions/*` を叩けます。

## Run

```bash
go run ./cmd/server
```

環境変数:

- `API_HTTP_ADDR` 既定値 `:8080`
- `APP_ENV` 既定値 `development`

## Example

```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"user@example.com\",\"password\":\"secret123\",\"display_name\":\"Aki\",\"agreed_to_terms\":true}"
```
