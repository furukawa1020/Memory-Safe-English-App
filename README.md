# Memory Safe English

# 記憶保持負荷配慮型の英語学習アプリを構築するためのモノレポです。

このリポジトリは、要件定義書をもとにした初期実装土台を提供します。目的は「保持しなくても処理できる英語体験」を、Flutter クライアント、Go API、Python 解析ワーカーの分担で実現することです。

## Repository Structure

```text
.
├─ apps/
│  └─ mobile/              # Flutter app
├─ services/
│  ├─ api/                 # Go REST API
│  └─ worker/              # Python NLP / speech worker
├─ infra/
│  ├─ docker-compose.yml   # Local dev dependencies
│  └─ postgres/init/       # Initial SQL
└─ docs/
   ├─ architecture.md
   ├─ api-outline.md
   ├─ data-model.md
   └─ mvp-roadmap.md
```

## Product Principle

開発判断で迷ったら、次の順で優先します。

1. 保持を要求していないか
2. 一画面一目的になっているか
3. 単語列ではなく意味単位で扱っているか
4. 正誤ではなく処理落ち点を取れているか
5. 長文生成より崩れない短文を支援しているか

## Initial Scope

初期リリースは以下を対象にします。

- 認証
- 初期診断
- Chunk Reader
- Memory-Safe Listening の基礎機能
- Speaking Builder の基礎機能
- 会話レスキュー
- 簡易分析ダッシュボード
- イベントログ基盤

## Recommended Build Order

1. `infra/` のローカル開発基盤を立てる
2. `services/api/` に OpenAPI 駆動で auth / sessions を実装する
3. `infra/postgres/init/001_init.sql` を起点に DB を作る
4. `services/worker/` に chunking / skeleton の HTTP interface を実装する
5. `apps/mobile/` に認証、ホーム、読む画面を先行実装する
6. API と worker の統合を進める

## Current Status

- Go API:
  auth, token refresh, sessions, event logging の最小実装あり
- Python worker:
  `/health`, `/analyze/chunks` の最小実装あり
- Infra:
  PostgreSQL / Redis のローカル compose あり
- Docs:
  architecture / data model / roadmap / OpenAPI あり

## Next Concrete Tasks

- Go API の PostgreSQL repository 実装
- worker の skeleton extraction と speech 分析追加
- Flutter アプリの雛形作成
- Chunk Reader 用コンテンツ取得 API 実装
- API と worker の接続実装

## Documentation

- [Architecture](docs/architecture.md)
- [API Outline](docs/api-outline.md)
- [Data Model](docs/data-model.md)
- [MVP Roadmap](docs/mvp-roadmap.md)
- [OpenAPI](services/api/openapi/openapi.yaml)
