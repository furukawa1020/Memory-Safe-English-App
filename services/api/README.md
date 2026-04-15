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

現時点では、標準ライブラリだけで起動できる最小 API 基盤を入れています。将来の PostgreSQL 実装へ差し替えやすいよう、`repository -> service -> handler` の責務分離を入れています。

含まれるエンドポイント:

- `GET /health`
- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/refresh`
- `GET /me`
- `POST /sessions/start`
- `POST /sessions/{id}/event`
- `POST /sessions/{id}/complete`

認証は email/password ベースです。`/auth/register` または `/auth/login` で返る `tokens.access_token` を `Authorization: Bearer <token>` として送ると `/me` や `/sessions/*` を叩けます。`tokens.refresh_token` は `POST /auth/refresh` に送ると新しい token pair を再発行できます。

## Design Notes

- `internal/repository`: 永続化のインターフェース
- `internal/service`: ユースケースと入力検証
- `internal/handlers`: HTTP 変換層
- `internal/httpx`: リクエスト / レスポンスの共通処理
- `internal/authctx`: 認証済みユーザーの request context 管理
- `internal/app`: DI とミドルウェアとサーバ組み立て
- `internal/store/memory`: 開発用インメモリ実装

この構成にしているので、次に PostgreSQL 実装を足すときは handler をほぼ触らずに進められます。

追加で入っている保守性向上ポイント:

- `context.Context` を repository / service に通している
- Go 標準 `ServeMux` のパターンルーティングを使用している
- `X-Request-ID` を自動付与してログとレスポンスに反映する
- protected route は Bearer token middleware で一元管理している
- `httptest` ベースの HTTP テストを追加している

追加で入っているセキュリティ対策:

- パスワードは PBKDF2-SHA256 でハッシュ化
- access / refresh token は HMAC 署名付き
- API 応答に基本的なセキュリティヘッダを付与
- production ではデフォルトの token secret を禁止
- パスワード長とハッシュ反復回数に下限を設けている

## Run

```bash
go run ./cmd/server
```

環境変数:

- `API_HTTP_ADDR` 既定値 `:8080`
- `APP_ENV` 既定値 `development`
- `AUTH_TOKEN_SECRET` 本番では必須
- `AUTH_ACCESS_TOKEN_TTL` 既定値 `15m`
- `AUTH_REFRESH_TOKEN_TTL` 既定値 `168h`
- `PASSWORD_HASH_ITERATIONS` 既定値 `120000`

## Example

```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"user@example.com\",\"password\":\"secret1234567\",\"display_name\":\"Aki\",\"agreed_to_terms\":true}"
```
