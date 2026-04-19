from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol

from app.analysis.models import AnalyzeTextInput
from app.models import ChunkingResult, SkeletonResult


class ChunkAnalyzer(Protocol):
    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult: ...


class SkeletonAnalyzer(Protocol):
    def extract(self, text: str, language: str = "en") -> SkeletonResult: ...


@dataclass(frozen=True, slots=True)
class AnalysisRoute:
    path: str
    audit_name: str
    operation: str


@dataclass(slots=True)
class AnalysisService:
    chunk_analyzer: ChunkAnalyzer
    skeleton_analyzer: SkeletonAnalyzer

    def analyze(self, operation: str, request: AnalyzeTextInput) -> ChunkingResult | SkeletonResult:
        if operation == "chunking":
            return self.chunk_analyzer.chunk_text(text=request.text, language=request.language)
        if operation == "skeleton":
            return self.skeleton_analyzer.extract(text=request.text, language=request.language)
        raise ValueError(f"unsupported analysis operation: {operation}")

    @staticmethod
    def routes() -> tuple[AnalysisRoute, ...]:
        return (
            AnalysisRoute(path="/analyze/chunks", audit_name="chunking_analyzed", operation="chunking"),
            AnalysisRoute(path="/analyze/skeleton", audit_name="skeleton_analyzed", operation="skeleton"),
        )
