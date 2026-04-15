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
