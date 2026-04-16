import json
from http import HTTPStatus
from http.client import HTTPConnection
from threading import Thread

from app.config import Settings
from app.runtime import create_server


def test_health_endpoint() -> None:
    server = create_server(
        Settings(host="127.0.0.1", port=0, max_words_per_chunk=6, require_api_key=True, api_keys=("test-key",))
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
        Settings(host="127.0.0.1", port=0, max_words_per_chunk=4, require_api_key=True, api_keys=("test-key",))
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
            headers={"Content-Type": "application/json", "X-Worker-Api-Key": "test-key"},
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
        Settings(host="127.0.0.1", port=0, max_words_per_chunk=6, require_api_key=True, api_keys=("test-key",))
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
            headers={"Content-Type": "application/json", "X-Worker-Api-Key": "test-key"},
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
        Settings(host="127.0.0.1", port=0, max_words_per_chunk=6, require_api_key=True, api_keys=("test-key",))
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
            headers={"Content-Type": "application/json", "X-Worker-Api-Key": "test-key"},
        )
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.REQUEST_ENTITY_TOO_LARGE
        assert payload["error"]["code"] == "body_too_large"
    finally:
        server.shutdown()
        server.server_close()
