# Mobile アプリ

`apps/mobile` は Memory Safe English の Flutter クライアントです。

現時点のモバイル実装は、主に次を対象にしています。

- メールアドレスとパスワードによる認証
- コンテンツ一覧表示
- `Normal` `Chunk` `Skeleton` `Assisted` の Reader モード
- 自由入力テキストの chunk 分析

## 構成

```text
apps/mobile
|- lib/
|  |- app/
|  |- config/
|  |- core/
|  `- features/
|- analysis_options.yaml
|- pubspec.yaml
`- README.md
```

## 前提条件

- Flutter SDK が `PATH` に入っていること
- Android Studio などで Android エミュレーターが使えること
- バックエンドスタックが `http://127.0.0.1:8070` の Rust proxy 経由で起動していること

確認には、リポジトリルートから次を使えます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev-doctor.ps1
```

## bootstrap

リポジトリルートから次を実行します。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1
```

このスクリプトは次を行います。

- Flutter の存在確認
- `flutter create .` による platform ファイル生成
- `flutter pub get`

Android だけ必要なら:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1 -AndroidOnly
```

## 実行

Android エミュレーター向けに手動で起動する場合:

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8070
```

`10.0.2.2` はエミュレーターからホストへ戻るためのアドレスで、`8070` は Rust proxy を指します。

## 補助スクリプト

リポジトリルートから、次の補助スクリプトも使えます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-android-emulator.ps1
powershell -ExecutionPolicy Bypass -File .\scripts\run-mobile.ps1 -StartEmulator
```

`run-mobile.ps1` は必要に応じて bootstrap を先に行い、その後 `flutter run` を実行します。

## 接続先

Flutter クライアントは、Go API を直接ではなく Rust proxy に向けます。

主に使う route:

- `/auth/login`
- `/auth/refresh`
- `/contents`
- `/analysis/chunks`
- `/ready`
- `/bootstrap/mobile`

起動時には `/bootstrap/mobile` を読んで readiness を確認し、認証済みなら `/auth/refresh` も使って access token を更新します。

## 注意

- `android/` や `ios/` は、`flutter create .` 実行前は存在しません
- ローカル開発では proxy 経由にしておくと、readiness と認証まわりの挙動がアプリと揃います
