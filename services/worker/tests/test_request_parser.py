from __future__ import annotations

import json
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler
from io import BytesIO

from app.http.request_parser import parse_analysis_request


class _StubHandler(BaseHTTPRequestHandler):
    def __init__(self, path: str, payload: bytes, headers: dict[str, str]) -> None:
        self.path = path
        self.headers = headers
        self.rfile = BytesIO(payload)
        self.client_address = ("127.0.0.1", 12345)


def test_parse_analysis_request_parses_valid_payload() -> None:
    payload = json.dumps({"text": "We propose a memory safe interface.", "language": "en"}).encode("utf-8")
    handler = _StubHandler(
        path="/analyze/chunks",
        payload=payload,
        headers={"Content-Type": "application/json", "Content-Length": str(len(payload)), "X-Worker-Api-Key": "test-key"},
    )

    request, error = parse_analysis_request(handler, max_body_bytes=1024)

    assert error is None
    assert request is not None
    assert request.route_operation == "chunking"
    assert request.payload.language == "en"


def test_parse_analysis_request_rejects_non_object_payload() -> None:
    payload = json.dumps(["invalid"]).encode("utf-8")
    handler = _StubHandler(
        path="/analyze/chunks",
        payload=payload,
        headers={"Content-Type": "application/json", "Content-Length": str(len(payload))},
    )

    request, error = parse_analysis_request(handler, max_body_bytes=1024)

    assert request is None
    assert error is not None
    assert error[0] == HTTPStatus.BAD_REQUEST
    assert error[1]["error"]["code"] == "invalid_json"


def test_parse_analysis_request_rejects_invalid_language_tag() -> None:
    payload = json.dumps({"text": "hello", "language": "日本語"}).encode("utf-8")
    handler = _StubHandler(
        path="/analyze/chunks",
        payload=payload,
        headers={"Content-Type": "application/json", "Content-Length": str(len(payload))},
    )

    request, error = parse_analysis_request(handler, max_body_bytes=1024)

    assert request is None
    assert error is not None
    assert error[0] == HTTPStatus.BAD_REQUEST
    assert error[1]["error"]["code"] == "invalid_request"
