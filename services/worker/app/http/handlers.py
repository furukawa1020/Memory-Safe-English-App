from __future__ import annotations

import json
import logging
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler
from typing import TYPE_CHECKING

from app.http.audit import audit_event
from app.http.guards import authenticate_request, enforce_rate_limit, verify_request_signature
from app.http.json_response import default_headers, encode_json, error_payload
from app.http.request_models import AnalysisRequest
from app.http.request_parser import parse_analysis_request

if TYPE_CHECKING:
    from app.application import Application

LOGGER = logging.getLogger(__name__)


class WorkerHTTPHandler:
    def __init__(self, app: "Application") -> None:
        self.app = app

    def handle_health(self) -> tuple[HTTPStatus, dict]:
        return HTTPStatus.OK, {"status": "ok", "service": "worker"}

    def handle_chunking(self, raw_body: bytes) -> tuple[HTTPStatus, dict]:
        parsed, error = self._parse_text_payload(raw_body)
        if error is not None:
            return error
        text, language = parsed
        result = self.app.chunking_service.chunk_text(text=text, language=language)
        return HTTPStatus.OK, result.to_dict()

    def handle_skeleton(self, raw_body: bytes) -> tuple[HTTPStatus, dict]:
        parsed, error = self._parse_text_payload(raw_body)
        if error is not None:
            return error
        text, language = parsed
        result = self.app.skeleton_service.extract(text=text, language=language)
        return HTTPStatus.OK, result.to_dict()

    def _parse_text_payload(self, raw_body: bytes) -> tuple[tuple[str, str] | None, tuple[HTTPStatus, dict] | None]:
        try:
            payload = json.loads(raw_body or b"{}")
        except json.JSONDecodeError:
            return None, (HTTPStatus.BAD_REQUEST, error_payload("invalid_json", "request body must be valid JSON"))

        text = str(payload.get("text", "")).strip()
        language = str(payload.get("language", "en")).strip() or "en"
        if not text:
            return None, (HTTPStatus.BAD_REQUEST, error_payload("invalid_request", "text is required"))
        if len(text) > self.app.settings.max_text_chars:
            return None, (
                HTTPStatus.REQUEST_ENTITY_TOO_LARGE,
                error_payload("text_too_large", "text exceeds allowed length"),
            )
        return (text, language), None


def create_request_handler(app: "Application") -> type[BaseHTTPRequestHandler]:
    transport = WorkerHTTPHandler(app)

    class RequestHandler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            if self.path != "/health":
                self._write_json(HTTPStatus.NOT_FOUND, error_payload("not_found", "route not found"))
                return

            status, payload = transport.handle_health()
            audit_event("health_checked", path=self.path, remote_ip=self.client_address[0], status=status)
            self._write_json(status, payload)

        def do_POST(self) -> None:  # noqa: N802
            if self.path not in {"/analyze/chunks", "/analyze/skeleton"}:
                self._write_json(HTTPStatus.NOT_FOUND, error_payload("not_found", "route not found"))
                return

            request, request_error = parse_analysis_request(self, max_body_bytes=app.settings.max_body_bytes)
            if request_error is not None:
                self._write_json(*request_error)
                return

            gate_error = self._enforce_request_guards(request)
            if gate_error is not None:
                self._write_json(*gate_error)
                return

            try:
                if self.path == "/analyze/chunks":
                    status, payload = transport.handle_chunking(request.raw_body)
                    audit_name = "chunking_analyzed"
                else:
                    status, payload = transport.handle_skeleton(request.raw_body)
                    audit_name = "skeleton_analyzed"
            except Exception:  # pragma: no cover - defensive fallback
                LOGGER.exception("worker request failed")
                audit_event("request_failed", request=request, status=HTTPStatus.INTERNAL_SERVER_ERROR)
                status, payload = HTTPStatus.INTERNAL_SERVER_ERROR, error_payload("internal_error", "internal server error")
            else:
                audit_event(audit_name, request=request, status=status)
            self._write_json(status, payload)

        def log_message(self, format: str, *args: object) -> None:  # noqa: A003
            LOGGER.info("%s - %s", self.address_string(), format % args)

        def _enforce_request_guards(self, request: AnalysisRequest) -> tuple[HTTPStatus, dict] | None:
            auth_error = authenticate_request(
                request,
                require_api_key=app.settings.require_api_key,
                api_keys=app.settings.api_keys,
            )
            if auth_error is not None:
                audit_event("auth_failed", request=request, reason=auth_error[1]["error"]["code"])
                return auth_error

            signature_error = verify_request_signature(
                request,
                require_request_signature=app.settings.require_request_signature,
                signature_keys=app.settings.signature_keys,
                signature=self.headers.get("X-Worker-Signature", ""),
                timestamp=self.headers.get("X-Worker-Timestamp", ""),
                max_age_seconds=app.settings.signature_max_age_seconds,
            )
            if signature_error is not None:
                audit_event("auth_failed", request=request, reason=signature_error[1]["error"]["code"])
                return signature_error

            rate_limit_error = enforce_rate_limit(request, limiter=app.rate_limiter)
            if rate_limit_error is not None:
                audit_event("rate_limited", request=request)
                return rate_limit_error
            return None

        def _write_json(self, status: HTTPStatus, payload: dict) -> None:
            body = encode_json(payload)
            self.send_response(status)
            for header_name, header_value in default_headers(body):
                self.send_header(header_name, header_value)
            self.end_headers()
            self.wfile.write(body)

    return RequestHandler
