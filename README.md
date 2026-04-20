# Memory Safe English

Memory Safe English は、ワーキングメモリ負荷が低い人でも英語を処理しやすくするための学習プロダクトです。  
読む・聞く・話す・会話レスキューを、`保持しなくても処理しやすい形` に変換することを目的にしています。

このリポジトリはモノレポで、主に次の 4 つで構成されています。

- Flutter モバイルアプリ
- Go API
- Python 分析ワーカー
- Rust プロキシ

## リポジトリ構成

```text
.
|- apps/
|  `- mobile/                 # Flutter アプリ
|- services/
|  |- api/                    # Go REST API
|  |- worker/                 # Python 分析ワーカー
|  `- proxy-rs/               # Rust プロキシ、キャッシュ、GC
|- infra/
|  |- docker-compose.yml      # ローカル開発用スタック
|  `- postgres/init/          # 初期 SQL
|- docs/
|  |- architecture.md
|  |- api-outline.md
|  |- data-model.md
|  `- mvp-roadmap.md
`- scripts/
   |- bootstrap-mobile.ps1
   |- dev-doctor.ps1
   |- run-mobile.ps1
   |- smoke-test.ps1
   |- start-android-emulator.ps1
   |- start-dev-stack.ps1
   `- stop-dev-stack.ps1
```

## 各サービスの役割

- `services/api`
  認証、コンテンツ配信、セッション管理、worker 連携
- `services/worker`
  chunking / skeleton / reader-plan / listening-plan / speaking-plan / rescue-plan / assessment / analytics
- `services/proxy-rs`
  前段プロキシ、フロント向け route 集約、認証系 rate limit、短期キャッシュ、readiness
- `apps/mobile`
  認証、コンテンツ一覧、Reader フローを持つ Flutter クライアント

## ローカル開発スタック

ローカルスタックは [infra/docker-compose.yml](./infra/docker-compose.yml) で定義しています。

- `proxy`: `http://127.0.0.1:8070`
- `api`: `http://127.0.0.1:8080`
- `worker`: `http://127.0.0.1:8090`
- `postgres`: `127.0.0.1:5432`
- `redis`: `127.0.0.1:6379`

すべてのサービスに health check を入れてあります。  
proxy は `/ready` を使うため、API と worker の両方が準備できてから healthy になります。

## 推奨ローカルワークフロー

### 1. バックエンドスタックを起動する

先に Docker Desktop を起動してください。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-dev-stack.ps1
```

このスクリプトは次を行います。

- Docker の利用可否を確認
- `docker compose up -d --build` を実行
- 全コンテナが healthy になるまで待機
- 既定では smoke test も実行

### 2. doctor で不足を確認する

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev-doctor.ps1
```

このスクリプトは次を確認します。

- Docker CLI
- Docker daemon
- Flutter SDK
- adb
- Android emulator
- AVD
- proxy readiness

### 3. Flutter アプリを bootstrap する

Flutter SDK はこのリポジトリに含まれていません。先に Flutter を入れてから実行してください。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1
```

このスクリプトは次を行います。

- Flutter の存在確認
- 必要なら `flutter create .` で platform ファイル生成
- `flutter pub get`

### 4. Android エミュレーターを起動する

Android Studio などで AVD を作成済みなら、次で起動できます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-android-emulator.ps1
```

複数の AVD がある場合は、先頭のものを既定で使います。  
指定したい場合は次のようにします。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-android-emulator.ps1 -AvdName Pixel_8_API_35
```

### 5. アプリをエミュレーターで起動する

手動で起動する場合:

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8070
```

`10.0.2.2` は Android エミュレーターからホスト側へ戻るためのアドレスです。  
`8070` は Rust proxy を指します。

補助スクリプトを使う場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-mobile.ps1 -StartEmulator
```

バックエンドも一緒に起動したい場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-mobile.ps1 -StartStack -StartEmulator
```

## smoke test

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\smoke-test.ps1
```

smoke test では次を確認します。

- proxy readiness
- mobile bootstrap metadata
- auth register / login
- content listing
- chunk analysis
- skeleton analysis
- proxy admin cache stats

## バックエンド検証

```bash
cd services/api && go test ./...
cd services/worker && python -m pytest tests -q
cd services/proxy-rs && cargo test
```

## ドキュメント

- [Architecture](./docs/architecture.md)
- [API Outline](./docs/api-outline.md)
- [Data Model](./docs/data-model.md)
- [MVP Roadmap](./docs/mvp-roadmap.md)
- [OpenAPI](./services/api/openapi/openapi.yaml)
