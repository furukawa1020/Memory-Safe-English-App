# Worker Service

Python 製の NLP / 音声解析ワーカーです。

初期実装では、依存導入で止まりにくいよう標準ライブラリ中心で組んでいます。今は `chunking` の最小価値に絞りつつ、あとから `skeleton` や `speech` を足しやすいように責務分離を入れています。

## Design

- `app/config.py`: 環境変数と設定管理
- `app/application.py`: サービスの組み立て
- `app/chunking/`: テキスト処理のユースケース
- `app/http/`: HTTP 入出力とレスポンス整形
- `app/runtime.py`: サーバ生成と起動
- `tests/`: chunking / runtime / HTTP のテスト

この形にしているので、HTTP を差し替えても chunking ロジックを保てますし、逆に chunking を入れ替えても transport 層は大きく崩れません。

## Current Features

- 英文の軽量チャンク分割
- チャンクの簡易 role 推定
- summary 生成の最小実装
- HTTP worker endpoint
- `Settings` ベースの構成管理
- pytest でのユニット / HTTP テスト

## Structure

```text
services/worker
├─ app/
│  ├─ chunking/
│  ├─ http/
│  ├─ application.py
│  ├─ config.py
│  ├─ models.py
│  ├─ runtime.py
│  └─ server.py
├─ tests/
└─ pyproject.toml
```

## Run

```bash
python -m app.server
```

既定では `127.0.0.1:8090` で起動します。

環境変数:

- `WORKER_HOST` 既定値 `127.0.0.1`
- `WORKER_PORT` 既定値 `8090`
- `CHUNKING_MAX_WORDS` 既定値 `6`

## API

### `GET /health`

worker の疎通確認です。

### `POST /analyze/chunks`

request:

```json
{
  "text": "In this study, we propose a memory safe interface.",
  "language": "en"
}
```

response:

```json
{
  "language": "en",
  "chunks": [
    {
      "order": 1,
      "text": "In this study",
      "role": "modifier",
      "skeleton_rank": 2
    }
  ],
  "summary": "In this study / we propose a memory safe"
}
```
