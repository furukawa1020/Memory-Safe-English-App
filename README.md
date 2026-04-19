# Memory Safe English

ワーキングメモリ負荷を下げながら英語を読んで、聞いて、話せるようにする学習アプリです。

このリポジトリはモノレポ構成で、Flutter クライアント、Go API、Python worker、Rust proxy、ローカル開発用 infra をまとめています。

## Repository Structure

```text
.
|- apps/
|  `- mobile/                 # Flutter app
|- services/
|  |- api/                    # Go REST API
|  |- worker/                 # Python analysis worker
|  `- proxy-rs/               # Rust proxy and cache GC
|- infra/
|  |- docker-compose.yml      # Local dev stack
|  `- postgres/init/          # Initial SQL
`- docs/
   |- architecture.md
   |- api-outline.md
   |- data-model.md
   `- mvp-roadmap.md
```

## Current Services

- `services/api`: auth, content, sessions, analysis orchestration
- `services/worker`: chunking and skeleton analysis with request signing
- `services/proxy-rs`: upstream proxy, short-lived response cache, cache GC, admin endpoints
- `apps/mobile`: Flutter client skeleton and reader flow

## Local Stack

`infra/docker-compose.yml` から以下を起動できます。

- `proxy` on `http://127.0.0.1:8070`
- `api` on `http://127.0.0.1:8080`
- `worker` on `http://127.0.0.1:8090`
- `postgres` on `127.0.0.1:5432`
- `redis` on `127.0.0.1:6379`

起動:

```bash
docker compose -f infra/docker-compose.yml up --build
```

## Documentation

- [Architecture](docs/architecture.md)
- [API Outline](docs/api-outline.md)
- [Data Model](docs/data-model.md)
- [MVP Roadmap](docs/mvp-roadmap.md)
- [OpenAPI](services/api/openapi/openapi.yaml)
