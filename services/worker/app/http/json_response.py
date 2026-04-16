from __future__ import annotations

import json
from http import HTTPStatus
from typing import Any


def encode_json(payload: dict[str, Any]) -> bytes:
    return json.dumps(payload).encode("utf-8")


def error_payload(code: str, message: str) -> dict[str, Any]:
    return {"error": {"code": code, "message": message}}


def default_headers(body: bytes) -> list[tuple[str, str]]:
    return [
        ("Content-Type", "application/json"),
        ("Content-Length", str(len(body))),
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "DENY"),
        ("Referrer-Policy", "no-referrer"),
        ("Content-Security-Policy", "default-src 'none'; frame-ancestors 'none'"),
    ]
