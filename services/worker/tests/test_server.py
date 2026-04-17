from __future__ import annotations

import json
from http import HTTPStatus
from http.client import HTTPConnection

from tests.conftest import make_settings, post_json, running_server, signed_headers


def test_health_endpoint() -> None:
    with running_server(make_settings()) as server:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        conn.request("GET", "/health")
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.OK
        assert payload["status"] == "ok"
        assert response.getheader("X-Content-Type-Options") == "nosniff"


def test_chunking_endpoint() -> None:
    with running_server(make_settings(max_words_per_chunk=4)) as server:
        body = {"text": "We propose a memory safe interface."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["chunks"]


def test_chunking_endpoint_rejects_empty_text() -> None:
    with running_server(make_settings()) as server:
        body = {"text": ""}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))

        assert response.status == HTTPStatus.BAD_REQUEST
        assert payload["error"]["code"] == "invalid_request"


def test_chunking_endpoint_requires_api_key() -> None:
    with running_server(make_settings()) as server:
        response, payload = post_json(
            server.server_port,
            "/analyze/chunks",
            {"text": "hello"},
            {"Content-Type": "application/json"},
        )

        assert response.status == HTTPStatus.UNAUTHORIZED
        assert payload["error"]["code"] == "unauthorized"


def test_chunking_endpoint_enforces_body_limit() -> None:
    with running_server(make_settings(max_body_bytes=16)) as server:
        body = {"text": "This body is intentionally too large."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))

        assert response.status == HTTPStatus.REQUEST_ENTITY_TOO_LARGE
        assert payload["error"]["code"] == "body_too_large"


def test_chunking_endpoint_requires_valid_signature() -> None:
    with running_server(make_settings()) as server:
        response, payload = post_json(
            server.server_port,
            "/analyze/chunks",
            {"text": "hello"},
            {
                "Content-Type": "application/json",
                "X-Worker-Api-Key": "test-key",
                "X-Worker-Timestamp": "1",
                "X-Worker-Signature": "bad-signature",
            },
        )

        assert response.status == HTTPStatus.UNAUTHORIZED
        assert payload["error"]["code"] == "invalid_signature"


def test_chunking_endpoint_rate_limits_burst_requests() -> None:
    with running_server(make_settings(rate_limit_max_requests=1, rate_limit_window_seconds=60)) as server:
        body = {"text": "hello"}
        body_text = json.dumps(body)
        first_response, _ = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))
        assert first_response.status == HTTPStatus.OK

        second_response, payload = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))
        assert second_response.status == HTTPStatus.TOO_MANY_REQUESTS
        assert payload["error"]["code"] == "rate_limited"
