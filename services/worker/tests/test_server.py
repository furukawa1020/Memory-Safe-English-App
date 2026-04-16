import json
import hmac
import hashlib
import time
from http import HTTPStatus
from http.client import HTTPConnection
from threading import Thread

from app.config import Settings
from app.runtime import create_server


def _signed_headers(body: str, api_key: str = "test-key", signing_key: str = "sign-key") -> dict[str, str]:
    timestamp = str(int(time.time()))
    signature = hmac.new(signing_key.encode("utf-8"), timestamp.encode("utf-8") + b"." + body.encode("utf-8"), hashlib.sha256).hexdigest()
    return {
        "Content-Type": "application/json",
        "X-Worker-Api-Key": api_key,
        "X-Worker-Timestamp": timestamp,
        "X-Worker-Signature": signature,
    }


def test_health_endpoint() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        conn.request("GET", "/health")
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.OK
        assert payload["status"] == "ok"
        assert response.getheader("X-Content-Type-Options") == "nosniff"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=4,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "We propose a memory safe interface."})
        conn.request(
            "POST",
            "/analyze/chunks",
            body=body,
            headers=_signed_headers(body),
        )
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.OK
        assert payload["chunks"]
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint_rejects_empty_text() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": ""})
        conn.request(
            "POST",
            "/analyze/chunks",
            body=body,
            headers=_signed_headers(body),
        )
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.BAD_REQUEST
        assert payload["error"]["code"] == "invalid_request"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint_requires_api_key() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "hello"})
        conn.request("POST", "/analyze/chunks", body=body, headers={"Content-Type": "application/json"})
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.UNAUTHORIZED
        assert payload["error"]["code"] == "unauthorized"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint_enforces_body_limit() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
            max_body_bytes=16,
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "This body is intentionally too large."})
        conn.request(
            "POST",
            "/analyze/chunks",
            body=body,
            headers=_signed_headers(body),
        )
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.REQUEST_ENTITY_TOO_LARGE
        assert payload["error"]["code"] == "body_too_large"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint_requires_valid_signature() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "hello"})
        conn.request(
            "POST",
            "/analyze/chunks",
            body=body,
            headers={
                "Content-Type": "application/json",
                "X-Worker-Api-Key": "test-key",
                "X-Worker-Timestamp": str(int(time.time())),
                "X-Worker-Signature": "bad-signature",
            },
        )
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.UNAUTHORIZED
        assert payload["error"]["code"] == "invalid_signature"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint_rate_limits_burst_requests() -> None:
    server = create_server(
        Settings(
            host="127.0.0.1",
            port=0,
            max_words_per_chunk=6,
            require_api_key=True,
            api_keys=("test-key",),
            require_request_signature=True,
            signature_keys=("sign-key",),
            rate_limit_max_requests=1,
            rate_limit_window_seconds=60,
        )
    )
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "hello"})
        conn.request("POST", "/analyze/chunks", body=body, headers=_signed_headers(body))
        first = conn.getresponse()
        first.read()
        assert first.status == HTTPStatus.OK

        conn = HTTPConnection("127.0.0.1", server.server_port)
        conn.request("POST", "/analyze/chunks", body=body, headers=_signed_headers(body))
        second = conn.getresponse()
        payload = json.loads(second.read())

        assert second.status == HTTPStatus.TOO_MANY_REQUESTS
        assert payload["error"]["code"] == "rate_limited"
    finally:
        server.shutdown()
        server.server_close()
