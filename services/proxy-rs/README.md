# Proxy Service

`services/proxy-rs` is the Rust front proxy for the local stack.

It sits in front of the Go API and Python worker and provides:

- shared request handling
- frontend-friendly API aliases at the proxy root
- readiness checks
- short-lived caching for worker analysis responses
- cache garbage collection
- admin cache inspection and purge endpoints
- mobile bootstrap metadata

## Routes

- `GET /health`: liveness
- `GET /ready`: readiness against API and worker upstreams
- `GET /bootstrap/mobile`: mobile-facing bootstrap metadata including recommended base URLs and route hints
- `/auth/*`, `/me`, `/analysis/*`, `/contents*`, `/sessions/*`: proxied to the Go API
- `/api/*`: proxied to the Go API
- `/worker/*`: proxied to the Python worker
- `GET /admin/cache`: cache stats
- `POST /admin/cache/purge`: cache purge

## Security and Operations

- request IDs are propagated with `X-Request-ID`
- admin endpoints require `X-Proxy-Admin-Token`
- cache responses include `X-Proxy-Cache`
- proxied responses include `X-Proxy-Upstream`
- responses include `X-Content-Type-Options`, `X-Frame-Options`, and `Referrer-Policy`

## Run

```bash
cargo run
```

## Container

```bash
docker build -t mse-proxy services/proxy-rs
```

## Verify

```bash
cargo test
```
