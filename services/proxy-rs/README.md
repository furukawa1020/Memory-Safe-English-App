# Rust Proxy Service

`services/proxy-rs` は Go API と Python worker の前段に置く Rust 製プロキシです。

目的:

- upstream 接続責務を一か所に集約する
- worker の分析レスポンスを短期キャッシュする
- 期限切れキャッシュをバックグラウンド GC で掃除する
- 管理 endpoint から cache 状態確認と purge を行えるようにする

## Structure

```text
services/proxy-rs
|- src/admin.rs
|- src/cache.rs
|- src/config.rs
|- src/gc.rs
|- src/proxy.rs
|- src/routes.rs
|- src/state.rs
|- src/main.rs
`- src/lib.rs
```

## Endpoints

- `GET /health`
- `GET /admin/cache`
- `POST /admin/cache/purge`
- `ANY /api/*path` -> Go API へ転送
- `ANY /worker/*path` -> Python worker へ転送

## Cache Policy

現状は worker の下記 POST endpoint だけをキャッシュします。

- `/analyze/chunks`
- `/analyze/skeleton`

キャッシュキーは `method + normalized path + sha256(body)` です。

## GC

- `PROXY_CACHE_TTL_SECONDS` を過ぎたエントリは失効
- `PROXY_GC_INTERVAL_SECONDS` ごとに sweep
- `PROXY_CACHE_MAX_ENTRIES` を超えた場合は古い順に整理

## Admin

管理 endpoint は `X-Proxy-Admin-Token` ヘッダで保護されます。

- `GET /admin/cache`: 現在のキャッシュ件数と期限情報を返す
- `POST /admin/cache/purge`: `{"scope":"all" | "chunks" | "skeleton"}` で purge

## Run

```bash
cargo run
```

主な環境変数:

- `PROXY_HTTP_ADDR` default `127.0.0.1:8070`
- `PROXY_API_BASE_URL` default `http://127.0.0.1:8080`
- `PROXY_WORKER_BASE_URL` default `http://127.0.0.1:8090`
- `PROXY_ADMIN_TOKEN` admin endpoint 保護用トークン
- `PROXY_UPSTREAM_TIMEOUT_SECONDS` default `10`
- `PROXY_CACHE_TTL_SECONDS` default `300`
- `PROXY_GC_INTERVAL_SECONDS` default `60`
- `PROXY_CACHE_MAX_ENTRIES` default `1024`
- `PROXY_MAX_REQUEST_BODY_BYTES` default `262144`

## Verify

```bash
cargo test
```
