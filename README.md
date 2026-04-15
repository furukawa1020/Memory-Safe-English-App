# Memory Safe English

記憶保持負荷配慮型の英語学習アプリを構築するためのモノレポです。

このリポジトリは、要件定義書をもとにした初期実装土台を提供します。目的は「保持しなくても処理できる英語体験」を、Flutter クライアント、Go API、Python 解析ワーカーの分担で実現することです。

## Repository Structure

```text
.
├─ apps/
│  └─ mobile/              # Flutter app
├─ services/
│  ├─ api/                 # Go REST API
│  └─ worker/              # Python NLP / speech workers
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
2. `services/api/` に OpenAPI 駆動で認証・セッション API を実装する
3. `infra/postgres/init/001_init.sql` を起点に DB を作る
4. `apps/mobile/` に認証、ホーム、読む画面を先行実装する
5. `services/worker/` にチャンク分割ロジックを実装する
6. イベントログを API とアプリの両方に組み込む

## Next Concrete Tasks

- Flutter アプリの雛形作成
- Go API のエントリポイントと OpenAPI 雛形追加
- Python worker の FastAPI もしくは job worker 雛形追加
- 認証 API と `users` / `user_profiles` 実装
- Chunk Reader 用コンテンツ取得 API 実装

## Documentation

- [Architecture](docs/architecture.md)
- [API Outline](docs/api-outline.md)
- [Data Model](docs/data-model.md)
- [MVP Roadmap](docs/mvp-roadmap.md)
