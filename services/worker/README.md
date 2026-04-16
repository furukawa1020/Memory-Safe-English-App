# Worker Service

Python 製の NLP / 音声解析ワーカーです。

初期実装では、依存導入で止まりにくいよう標準ライブラリ中心で組んでいます。今は `chunking` の最小価値に絞りつつ、あとから `skeleton` や `speech` を足しやすいように責務分離を入れています。

先に進める前提として、worker には最低限のセキュリティ対策も入れています。

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

## Security

- `X-Worker-Api-Key` による API key 認証
- 認証必須を `WORKER_REQUIRE_API_KEY` で制御
- `X-Worker-Timestamp` と `X-Worker-Signature` による HMAC 署名検証
- 署名時刻の許容幅を `WORKER_SIGNATURE_MAX_AGE_SECONDS` で制御
- インメモリ sliding window rate limiting
- `WORKER_MAX_BODY_BYTES` による body サイズ制限
- `WORKER_MAX_TEXT_CHARS` による入力テキスト長制限
- `application/json` 以外を拒否
- 接続ソケットに request timeout を設定
- `nosniff`, `DENY`, `no-referrer`, `CSP` の基本ヘッダを付与
- 内部例外は汎用的な `internal_error` に丸めて返却
- JSON ベースの監査ログ出力

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
- `WORKER_REQUIRE_API_KEY` 既定値 `true`
- `WORKER_API_KEYS` カンマ区切りの API key 一覧
- `WORKER_REQUIRE_REQUEST_SIGNATURE` 既定値 `true`
- `WORKER_SIGNATURE_KEYS` カンマ区切りの HMAC 署名鍵
- `WORKER_SIGNATURE_MAX_AGE_SECONDS` 既定値 `300`
- `WORKER_MAX_BODY_BYTES` 既定値 `16384`
- `WORKER_MAX_TEXT_CHARS` 既定値 `4000`
- `WORKER_REQUEST_TIMEOUT_SECONDS` 既定値 `10`
- `WORKER_RATE_LIMIT_MAX_REQUESTS` 既定値 `30`
- `WORKER_RATE_LIMIT_WINDOW_SECONDS` 既定値 `60`

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

headers:

```text
Content-Type: application/json
X-Worker-Api-Key: <shared-secret>
X-Worker-Timestamp: <unix-seconds>
X-Worker-Signature: <hex-hmac-sha256(timestamp + "." + raw-body)>
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
