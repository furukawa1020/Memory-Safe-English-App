from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol

from app.analysis.models import AnalyzeTextInput
from app.models import ChunkingResult, ListeningPlanResult, ReaderPlanResult, SkeletonResult, SpeakingPlanResult


class ChunkAnalyzer(Protocol):
    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult: ...


class SkeletonAnalyzer(Protocol):
    def extract(self, text: str, language: str = "en") -> SkeletonResult: ...


class ReaderPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en") -> ReaderPlanResult: ...


class ListeningPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en") -> ListeningPlanResult: ...


class SpeakingPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en") -> SpeakingPlanResult: ...


@dataclass(frozen=True, slots=True)
class AnalysisRoute:
    path: str
    audit_name: str
    operation: str


@dataclass(slots=True)
class AnalysisService:
    chunk_analyzer: ChunkAnalyzer
    skeleton_analyzer: SkeletonAnalyzer
    reader_plan_analyzer: ReaderPlanAnalyzer
    listening_plan_analyzer: ListeningPlanAnalyzer
    speaking_plan_analyzer: SpeakingPlanAnalyzer

    def analyze(self, operation: str, request: AnalyzeTextInput) -> ChunkingResult | SkeletonResult | ReaderPlanResult | ListeningPlanResult | SpeakingPlanResult:
        if operation == "chunking":
            return self.chunk_analyzer.chunk_text(text=request.text, language=request.language)
        if operation == "skeleton":
            return self.skeleton_analyzer.extract(text=request.text, language=request.language)
        if operation == "reader_plan":
            return self.reader_plan_analyzer.build(text=request.text, language=request.language)
        if operation == "listening_plan":
            return self.listening_plan_analyzer.build(text=request.text, language=request.language)
        if operation == "speaking_plan":
            return self.speaking_plan_analyzer.build(text=request.text, language=request.language)
        raise ValueError(f"unsupported analysis operation: {operation}")

    @staticmethod
    def routes() -> tuple[AnalysisRoute, ...]:
        return (
            AnalysisRoute(path="/analyze/chunks", audit_name="chunking_analyzed", operation="chunking"),
            AnalysisRoute(path="/analyze/skeleton", audit_name="skeleton_analyzed", operation="skeleton"),
            AnalysisRoute(path="/analyze/reader-plan", audit_name="reader_plan_analyzed", operation="reader_plan"),
            AnalysisRoute(path="/analyze/listening-plan", audit_name="listening_plan_analyzed", operation="listening_plan"),
            AnalysisRoute(path="/analyze/speaking-plan", audit_name="speaking_plan_analyzed", operation="speaking_plan"),
        )
