from __future__ import annotations

from http import HTTPStatus
from http.server import BaseHTTPRequestHandler

from app.http.json_response import error_payload
from app.http.request_models import AnalysisRequest, RequestMetadata


def parse_analysis_request(handler: BaseHTTPRequestHandler, *, max_body_bytes: int) -> tuple[AnalysisRequest | None, tuple[HTTPStatus, dict] | None]:
    content_type = handler.headers.get("Content-Type", "")
    if "application/json" not in content_type.lower():
        return None, (
            HTTPStatus.UNSUPPORTED_MEDIA_TYPE,
            error_payload("unsupported_media_type", "Content-Type must be application/json"),
        )

    try:
        content_length = int(handler.headers.get("Content-Length", "0"))
    except ValueError:
        return None, (
            HTTPStatus.BAD_REQUEST,
            error_payload("invalid_content_length", "Content-Length must be a valid integer"),
        )

    if content_length <= 0:
        return None, (HTTPStatus.BAD_REQUEST, error_payload("invalid_request", "request body is required"))
    if content_length > max_body_bytes:
        return None, (
            HTTPStatus.REQUEST_ENTITY_TOO_LARGE,
            error_payload("body_too_large", "request body exceeds allowed size"),
        )

    raw_body = handler.rfile.read(content_length)
    metadata = RequestMetadata(
        path=handler.path,
        remote_ip=handler.client_address[0],
        api_key=handler.headers.get("X-Worker-Api-Key", ""),
    )
    return AnalysisRequest(metadata=metadata, raw_body=raw_body, content_type=content_type), None
