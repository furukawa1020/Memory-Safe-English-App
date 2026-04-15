from __future__ import annotations

import json
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from app.chunking import ChunkingService


class WorkerHandler(BaseHTTPRequestHandler):
    chunking_service = ChunkingService()

    def do_GET(self) -> None:  # noqa: N802
        if self.path != "/health":
            self._write_json(HTTPStatus.NOT_FOUND, {"error": {"code": "not_found", "message": "route not found"}})
            return

        self._write_json(HTTPStatus.OK, {"status": "ok", "service": "worker"})

    def do_POST(self) -> None:  # noqa: N802
        if self.path != "/analyze/chunks":
            self._write_json(HTTPStatus.NOT_FOUND, {"error": {"code": "not_found", "message": "route not found"}})
            return

        content_length = int(self.headers.get("Content-Length", "0"))
        raw_body = self.rfile.read(content_length)
        try:
            payload = json.loads(raw_body or b"{}")
        except json.JSONDecodeError:
            self._write_json(HTTPStatus.BAD_REQUEST, {"error": {"code": "invalid_json", "message": "request body must be valid JSON"}})
            return

        text = str(payload.get("text", "")).strip()
        language = str(payload.get("language", "en")).strip() or "en"
        if not text:
            self._write_json(HTTPStatus.BAD_REQUEST, {"error": {"code": "invalid_request", "message": "text is required"}})
            return

        result = self.chunking_service.chunk_text(text=text, language=language)
        self._write_json(HTTPStatus.OK, result.to_dict())

    def log_message(self, format: str, *args: object) -> None:  # noqa: A003
        return

    def _write_json(self, status: HTTPStatus, payload: dict) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def run(addr: str = "127.0.0.1", port: int = 8090) -> None:
    server = ThreadingHTTPServer((addr, port), WorkerHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    run()
