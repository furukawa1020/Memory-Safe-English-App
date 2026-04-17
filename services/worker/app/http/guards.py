from __future__ import annotations

import time
from http import HTTPStatus

from app.http.json_response import error_payload
from app.http.request_models import AnalysisRequest
from app.security import has_valid_api_key, has_valid_signature


def authenticate_request(request: AnalysisRequest, *, require_api_key: bool, api_keys: tuple[str, ...]) -> tuple[HTTPStatus, dict] | None:
    if not require_api_key:
        return None
    if not has_valid_api_key(request.metadata.api_key, api_keys):
        return HTTPStatus.UNAUTHORIZED, error_payload("unauthorized", "valid X-Worker-Api-Key header is required")
    return None


def verify_request_signature(
    request: AnalysisRequest,
    *,
    require_request_signature: bool,
    signature_keys: tuple[str, ...],
    signature: str,
    timestamp: str,
    max_age_seconds: int,
) -> tuple[HTTPStatus, dict] | None:
    if not require_request_signature:
        return None
    if not has_valid_signature(
        body=request.raw_body,
        provided_signature=signature,
        timestamp=timestamp,
        allowed_keys=signature_keys,
        max_age_seconds=max_age_seconds,
        now=int(time.time()),
    ):
        return HTTPStatus.UNAUTHORIZED, error_payload(
            "invalid_signature",
            "valid X-Worker-Timestamp and X-Worker-Signature headers are required",
        )
    return None


def enforce_rate_limit(request: AnalysisRequest, *, limiter) -> tuple[HTTPStatus, dict] | None:
    rate_limit_key = f"{request.metadata.remote_ip}:{request.metadata.api_key}"
    if limiter.allow(rate_limit_key):
        return None
    return HTTPStatus.TOO_MANY_REQUESTS, error_payload("rate_limited", "too many requests")
