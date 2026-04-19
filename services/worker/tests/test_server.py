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
        assert payload["version"]


def test_chunking_endpoint_rejects_empty_text() -> None:
    with running_server(make_settings()) as server:
        body = {"text": ""}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/chunks", body, signed_headers(body_text))

        assert response.status == HTTPStatus.BAD_REQUEST
        assert payload["error"]["code"] == "invalid_request"


def test_chunking_endpoint_rejects_non_object_payload() -> None:
    with running_server(make_settings()) as server:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        raw_body = json.dumps(["invalid"])
        conn.request("POST", "/analyze/chunks", body=raw_body, headers=signed_headers(raw_body))
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.BAD_REQUEST
        assert payload["error"]["code"] == "invalid_json"


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


def test_skeleton_endpoint() -> None:
    with running_server(make_settings()) as server:
        body = {"text": "In this study, we propose a memory safe interface."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/skeleton", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["parts"]
        assert payload["summary"]
        assert payload["version"]


def test_reader_plan_endpoint() -> None:
    with running_server(make_settings()) as server:
        body = {"text": "In this study, we propose a memory safe interface that reduces overload during reading."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/reader-plan", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["recommended_mode"] == "progressive"
        assert payload["display_strategy"]
        assert payload["focus_steps"]
        assert "overload_risk" in payload["focus_steps"][0]
        assert "presentation_hint" in payload["focus_steps"][0]
        assert "hotspots" in payload
        assert payload["version"]


def test_listening_plan_endpoint() -> None:
    with running_server(make_settings()) as server:
        body = {"text": "In this study, we propose a memory safe interface that reduces overload during reading."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/listening-plan", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["recommended_speed"]
        assert payload["pause_points"]
        assert payload["final_pass_strategy"]
        assert payload["version"]


def test_speaking_plan_endpoint() -> None:
    with running_server(make_settings()) as server:
        body = {"text": "In this study, we propose a memory safe interface that reduces overload during reading."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/speaking-plan", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["recommended_style"] == "short-linked-sentences"
        assert payload["steps"]
        assert payload["rescue_phrases"]
        assert payload["version"]


def test_rescue_plan_endpoint() -> None:
    with running_server(make_settings()) as server:
        body = {"text": "In this study, we propose a memory safe interface that reduces overload during reading."}
        body_text = json.dumps(body)
        response, payload = post_json(server.server_port, "/analyze/rescue-plan", body, signed_headers(body_text))

        assert response.status == HTTPStatus.OK
        assert payload["overload_level"]
        assert payload["primary_strategy"]
        assert payload["phrases"]
        assert payload["version"]
