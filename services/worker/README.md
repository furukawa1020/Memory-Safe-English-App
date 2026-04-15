# Worker Service

Python 製の NLP / 音声解析ワーカーです。

初期実装では、依存導入で止まりにくいよう標準ライブラリ中心で組んでいます。まずは API との接続境界を固めるため、`/health` と `/analyze/chunks` を提供します。

## Current Features

- 英文の軽量チャンク分割
- チャンクの簡易 role 推定
- summary 生成の最小実装
- HTTP worker endpoint
- unittest / pytest 互換で動くテスト

## Structure

```text
services/worker
├─ app/
│  ├─ chunking/
│  ├─ models.py
│  └─ server.py
├─ tests/
└─ pyproject.toml
```

## Run

```bash
python -m app.server
```

既定では `127.0.0.1:8090` で起動します。

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
