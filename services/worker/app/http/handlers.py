from __future__ import annotations

import json
from dataclasses import dataclass
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler
from typing import TYPE_CHECKING

from app.http.json_response import default_headers, encode_json, error_payload

if TYPE_CHECKING:
    from app.application import Application


@dataclass(slots=True)
class WorkerHTTPHandler:
    app: "Application"

    def handle_health(self) -> tuple[HTTPStatus, dict]:
        return HTTPStatus.OK, {"status": "ok", "service": "worker"}

    def handle_chunking(self, raw_body: bytes) -> tuple[HTTPStatus, dict]:
        try:
            payload = json.loads(raw_body or b"{}")
        except json.JSONDecodeError:
            return HTTPStatus.BAD_REQUEST, error_payload("invalid_json", "request body must be valid JSON")

        text = str(payload.get("text", "")).strip()
        language = str(payload.get("language", "en")).strip() or "en"
        if not text:
            return HTTPStatus.BAD_REQUEST, error_payload("invalid_request", "text is required")

        result = self.app.chunking_service.chunk_text(text=text, language=language)
        return HTTPStatus.OK, result.to_dict()


def create_request_handler(app: "Application") -> type[BaseHTTPRequestHandler]:
    transport = WorkerHTTPHandler(app)

    class RequestHandler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            if self.path != "/health":
                self._write_json(HTTPStatus.NOT_FOUND, error_payload("not_found", "route not found"))
                return

            status, payload = transport.handle_health()
            self._write_json(status, payload)

        def do_POST(self) -> None:  # noqa: N802
            if self.path != "/analyze/chunks":
                self._write_json(HTTPStatus.NOT_FOUND, error_payload("not_found", "route not found"))
                return

            content_length = int(self.headers.get("Content-Length", "0"))
            raw_body = self.rfile.read(content_length)
            status, payload = transport.handle_chunking(raw_body)
            self._write_json(status, payload)

        def log_message(self, format: str, *args: object) -> None:  # noqa: A003
            return

        def _write_json(self, status: HTTPStatus, payload: dict) -> None:
            body = encode_json(payload)
            self.send_response(status)
            for header_name, header_value in default_headers(body):
                self.send_header(header_name, header_value)
            self.end_headers()
            self.wfile.write(body)

    return RequestHandler
