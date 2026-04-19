# Proxy Service

`services/proxy-rs` is the Rust front proxy for the local stack.

It sits in front of the Go API and Python worker and provides:

- shared request handling
- frontend-friendly API aliases at the proxy root
- coarse auth route rate limiting before requests reach the API
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
- auth routes return `429` with `Retry-After` when the proxy-side limiter is exceeded
- responses include `X-Content-Type-Options`, `X-Frame-Options`, and `Referrer-Policy`
- when missing, `X-Forwarded-For` is added before forwarding to the API
- incoming forwarding headers are only trusted when the immediate peer IP is listed in `PROXY_TRUSTED_PROXY_IPS`
- `PROXY_ADMIN_TOKEN` must be at least 16 characters long

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
