import json
from http import HTTPStatus
from http.client import HTTPConnection
from http.server import ThreadingHTTPServer
from threading import Thread

from app.server import WorkerHandler


def test_health_endpoint() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", 0), WorkerHandler)
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        conn.request("GET", "/health")
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.OK
        assert payload["status"] == "ok"
    finally:
        server.shutdown()
        server.server_close()


def test_chunking_endpoint() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", 0), WorkerHandler)
    thread = Thread(target=server.serve_forever, daemon=True)
    thread.start()

    try:
        conn = HTTPConnection("127.0.0.1", server.server_port)
        body = json.dumps({"text": "We propose a memory safe interface."})
        conn.request("POST", "/analyze/chunks", body=body, headers={"Content-Type": "application/json"})
        response = conn.getresponse()
        payload = json.loads(response.read())

        assert response.status == HTTPStatus.OK
        assert payload["chunks"]
    finally:
        server.shutdown()
        server.server_close()
