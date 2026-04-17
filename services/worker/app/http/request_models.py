from __future__ import annotations

from dataclasses import dataclass


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
