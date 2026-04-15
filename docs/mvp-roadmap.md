# MVP Roadmap

## Phase 0: Foundation

目的:

- 実装の前提を揃える
- API / DB / イベントの責務境界を先に固定する

成果物:

- リポジトリ構成
- DB 初期スキーマ
- API アウトライン
- アーキテクチャ文書

## Phase 1: Core Platform

目的:

- ログインして学習を始められる状態を作る

対象:

- 認証
- ユーザープロファイル
- 設定
- セッション開始 / 完了
- イベントログ

Definition of Done:

- ユーザー登録 / ログイン可能
- 設定取得 / 更新可能
- セッション生成可能
- イベント送信可能

## Phase 2: Chunk Reader MVP

目的:

- 最初の中核価値を出す

対象:

- コンテンツ一覧 / 詳細
- チャンク表示
- Skeleton / Assisted モード
- 理解自己評価

Definition of Done:

- 英文コンテンツをチャンク単位で読める
- 骨格表示へ切替できる
- チャンクごとの滞在時間を記録できる

## Phase 3: Listening MVP

目的:

- 通し音声ではなく短区間理解を成立させる

対象:

- 区間再生
- 停止点挿入
- 速度変更
- 区間リピート
- テキスト後出し

Definition of Done:

- 短区間再生ができる
- 再生イベントが記録される
- 理解自己評価が取得できる

## Phase 4: Speaking Builder MVP

目的:

- 崩れない短文発話を支える

対象:

- 自己紹介テンプレ
- 研究紹介テンプレ
- 録音
- 簡易フィードバック

Definition of Done:

- テンプレを選んで短文練習できる
- 録音と試行ログが保存される
- 詰まり位置の簡易指標が返る

## Phase 5: Analytics MVP

目的:

- 正答率ではなく処理落ち点を見せる

対象:

- 今日の傾向
- 週次の変化
- 苦手パターン
- 次回学習のおすすめ

Definition of Done:

- 読む / 聞く / 話すの負荷傾向を返せる
- ユーザーが次回の学習候補を見られる

## Suggested First Sprint

1. Postgres 初期 schema を確定
2. Go API に `auth`, `me`, `sessions`, `events` を実装
3. Flutter でログイン、ホーム、読む画面を仮実装
4. Python worker にダミーの chunking interface を作る
5. E2E で「ログイン -> コンテンツ取得 -> セッション開始 -> イベント送信」まで通す
