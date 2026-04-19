from __future__ import annotations

from dataclasses import dataclass

from app.analysis import AnalyzeTextInput


@dataclass(slots=True)
class RequestMetadata:
    path: str
    remote_ip: str
    api_key: str


@dataclass(slots=True)
class AnalysisRequest:
    metadata: RequestMetadata
    raw_body: bytes
    content_type: str
    route_operation: str
    audit_name: str
    payload: AnalyzeTextInput
