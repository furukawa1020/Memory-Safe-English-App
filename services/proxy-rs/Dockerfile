FROM rust:1.87 AS builder
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN useradd --system --uid 10001 proxyuser
COPY --from=builder /app/target/release/mse-proxy /usr/local/bin/mse-proxy

USER proxyuser
EXPOSE 8070

ENTRYPOINT ["/usr/local/bin/mse-proxy"]
