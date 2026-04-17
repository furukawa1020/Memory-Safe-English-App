from __future__ import annotations

from http import HTTPStatus

from app.http.request_models import AnalysisRequest
from app.observability import audit_log


def audit_event(
    name: str,
    *,
    request: AnalysisRequest | None = None,
    path: str | None = None,
    remote_ip: str | None = None,
    status: HTTPStatus | None = None,
    **fields: object,
) -> None:
    payload: dict[str, object] = dict(fields)
    if request is not None:
        payload.setdefault("path", request.metadata.path)
        payload.setdefault("remote_ip", request.metadata.remote_ip)
    else:
        if path is not None:
            payload.setdefault("path", path)
        if remote_ip is not None:
            payload.setdefault("remote_ip", remote_ip)
    if status is not None:
        payload["status"] = status.value
    audit_log(name, **payload)
