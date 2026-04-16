from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler
from typing import TYPE_CHECKING

from app.http.json_response import default_headers, encode_json, error_payload
from app.security import has_valid_api_key

if TYPE_CHECKING:
    from app.application import Application

LOGGER = logging.getLogger(__name__)


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
        if len(text) > self.app.settings.max_text_chars:
            return HTTPStatus.REQUEST_ENTITY_TOO_LARGE, error_payload("text_too_large", "text exceeds allowed length")

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

             # Only accept authenticated JSON requests for analysis endpoints.
            auth_error = self._authenticate()
            if auth_error is not None:
                self._write_json(*auth_error)
                return

            content_type = self.headers.get("Content-Type", "")
            if "application/json" not in content_type.lower():
                self._write_json(HTTPStatus.UNSUPPORTED_MEDIA_TYPE, error_payload("unsupported_media_type", "Content-Type must be application/json"))
                return

            try:
                content_length = int(self.headers.get("Content-Length", "0"))
            except ValueError:
                self._write_json(HTTPStatus.BAD_REQUEST, error_payload("invalid_content_length", "Content-Length must be a valid integer"))
                return
            if content_length <= 0:
                self._write_json(HTTPStatus.BAD_REQUEST, error_payload("invalid_request", "request body is required"))
                return
            if content_length > app.settings.max_body_bytes:
                self._write_json(HTTPStatus.REQUEST_ENTITY_TOO_LARGE, error_payload("body_too_large", "request body exceeds allowed size"))
                return

            raw_body = self.rfile.read(content_length)
            try:
                status, payload = transport.handle_chunking(raw_body)
            except Exception:  # pragma: no cover - defensive fallback
                LOGGER.exception("worker request failed")
                status, payload = HTTPStatus.INTERNAL_SERVER_ERROR, error_payload("internal_error", "internal server error")
            self._write_json(status, payload)

        def log_message(self, format: str, *args: object) -> None:  # noqa: A003
            LOGGER.info("%s - %s", self.address_string(), format % args)

        def _authenticate(self) -> tuple[HTTPStatus, dict] | None:
            if not app.settings.require_api_key:
                return None

            api_key = self.headers.get("X-Worker-Api-Key", "")
            if not has_valid_api_key(api_key, app.settings.api_keys):
                return HTTPStatus.UNAUTHORIZED, error_payload("unauthorized", "valid X-Worker-Api-Key header is required")
            return None

        def _write_json(self, status: HTTPStatus, payload: dict) -> None:
            body = encode_json(payload)
            self.send_response(status)
            for header_name, header_value in default_headers(body):
                self.send_header(header_name, header_value)
            self.end_headers()
            self.wfile.write(body)

    return RequestHandler
