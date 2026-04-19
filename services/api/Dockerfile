FROM golang:1.24 AS builder
WORKDIR /app

COPY go.mod go.mod
COPY cmd cmd
COPY internal internal

RUN go build -o /out/api-server ./cmd/server

FROM debian:bookworm-slim
WORKDIR /app

RUN useradd --system --uid 10001 apiuser
COPY --from=builder /out/api-server /usr/local/bin/api-server

USER apiuser
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/api-server"]
