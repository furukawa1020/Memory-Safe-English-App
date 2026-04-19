# Mobile App

Flutter クライアントです。認証、content 一覧、Chunk Reader、自由入力の chunk analysis を扱います。

## Structure

```text
apps/mobile
|- lib/
|  |- app/
|  |- config/
|  |- core/
|  `- features/
|- test/
|- analysis_options.yaml
`- pubspec.yaml
```

## Current Features

- email/password の login / register
- content 一覧の取得
- content 詳細と `Normal / Chunk / Skeleton / Assisted` の表示切り替え
- 自由入力テキストの chunk analysis

## Run

```bash
flutter pub get
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8080
```
