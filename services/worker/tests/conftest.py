from __future__ import annotations

import hashlib
import hmac
import json
import time
from collections.abc import Iterator
from contextlib import contextmanager
from http.client import HTTPConnection
from threading import Thread

from app.config import Settings
from app.runtime import create_server


def signed_headers(body: str, api_key: str = "test-key", signing_key: str = "sign-key") -> dict[str, str]:
    timestamp = str(int(time.time()))
    signature = hmac.new(signing_key.encode("utf-8"), timestamp.encode("utf-8") + b"." + body.encode("utf-8"), hashlib.sha256).hexdigest()
    return {
        "Content-Type": "application/json",
        "X-Worker-Api-Key": api_key,
        "X-Worker-Timestamp": timestamp,
        "X-Worker-Signature": signature,
    }


def test_settings(**overrides: object) -> Settings:
    defaults: dict[str, object] = {
        "host": "127.0.0.1",
        "port": 0,
        "max_words_per_chunk": 6,
        "require_api_key": True,
        "api_keys": ("test-key",),
        "require_request_signature": True,
        "signature_keys": ("sign-key",),
    }
    defaults.update(overrides)
    return Settings(**defaults)


@contextmanager
def running_server(settings: Settings) -> Iterator[object]:
    server = create_server(settings)
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield server
    finally:
        server.shutdown()
        server.server_close()


def post_json(port: int, path: str, payload: dict, headers: dict[str, str]) -> tuple[object, dict]:
    conn = HTTPConnection("127.0.0.1", port)
    body = json.dumps(payload)
    conn.request("POST", path, body=body, headers=headers)
    response = conn.getresponse()
    parsed = json.loads(response.read())
    return response, parsed
