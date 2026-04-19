from __future__ import annotations

from dataclasses import dataclass

from app.analysis.service import AnalysisService


@dataclass(frozen=True, slots=True)
class RouteMatch:
    operation: str
    audit_name: str


ANALYSIS_ROUTES = {route.path: RouteMatch(operation=route.operation, audit_name=route.audit_name) for route in AnalysisService.routes()}
