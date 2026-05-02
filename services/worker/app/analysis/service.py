from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol

from app.analysis.models import AnalyzeTextInput
from app.models import (
    AssessmentProfileResult,
    AnalyticsSummaryResult,
    CollapsePatternResult,
    ChunkingResult,
    ListeningPlanResult,
    PracticeSetResult,
    ReaderPlanResult,
    RescuePlanResult,
    SkeletonResult,
    SpeakingPlanResult,
)


class ChunkAnalyzer(Protocol):
    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult: ...


class SkeletonAnalyzer(Protocol):
    def extract(self, text: str, language: str = "en") -> SkeletonResult: ...


class ReaderPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en", target_context: str = "general") -> ReaderPlanResult: ...


class ListeningPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en", target_context: str = "general") -> ListeningPlanResult: ...


class SpeakingPlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en", target_context: str = "general") -> SpeakingPlanResult: ...


class RescuePlanAnalyzer(Protocol):
    def build(self, text: str, language: str = "en", target_context: str = "general") -> RescuePlanResult: ...


class AssessmentAnalyzer(Protocol):
    def assess(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
    ) -> AssessmentProfileResult: ...


class CollapsePatternAnalyzer(Protocol):
    def analyze(
        self,
        text: str,
        *,
        language: str = "en",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> CollapsePatternResult: ...


class AnalyticsSummaryAnalyzer(Protocol):
    def summarize(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> AnalyticsSummaryResult: ...


class PracticeSetAnalyzer(Protocol):
    def build(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> PracticeSetResult: ...


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
    rescue_plan_analyzer: RescuePlanAnalyzer
    assessment_analyzer: AssessmentAnalyzer
    collapse_pattern_analyzer: CollapsePatternAnalyzer
    analytics_summary_analyzer: AnalyticsSummaryAnalyzer
    practice_set_analyzer: PracticeSetAnalyzer

    def analyze(self, operation: str, request: AnalyzeTextInput) -> ChunkingResult | SkeletonResult | ReaderPlanResult | ListeningPlanResult | SpeakingPlanResult | RescuePlanResult | AssessmentProfileResult | CollapsePatternResult | AnalyticsSummaryResult | PracticeSetResult:
        if operation == "chunking":
            return self.chunk_analyzer.chunk_text(text=request.text, language=request.language)
        if operation == "skeleton":
            return self.skeleton_analyzer.extract(text=request.text, language=request.language)
        if operation == "reader_plan":
            return self.reader_plan_analyzer.build(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
            )
        if operation == "listening_plan":
            return self.listening_plan_analyzer.build(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
            )
        if operation == "speaking_plan":
            return self.speaking_plan_analyzer.build(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
            )
        if operation == "rescue_plan":
            return self.rescue_plan_analyzer.build(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
            )
        if operation == "assessment":
            return self.assessment_analyzer.assess(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
                self_reported_difficulties=request.self_reported_difficulties,
                fatigue_level=request.fatigue_level,
            )
        if operation == "collapse_patterns":
            return self.collapse_pattern_analyzer.analyze(
                text=request.text,
                language=request.language,
                session_events=request.session_events,
            )
        if operation == "analytics_summary":
            return self.analytics_summary_analyzer.summarize(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
                self_reported_difficulties=request.self_reported_difficulties,
                fatigue_level=request.fatigue_level,
                session_events=request.session_events,
            )
        if operation == "practice_set":
            return self.practice_set_analyzer.build(
                text=request.text,
                language=request.language,
                target_context=request.target_context,
                self_reported_difficulties=request.self_reported_difficulties,
                fatigue_level=request.fatigue_level,
                session_events=request.session_events,
            )
        raise ValueError(f"unsupported analysis operation: {operation}")

    @staticmethod
    def routes() -> tuple[AnalysisRoute, ...]:
        return (
            AnalysisRoute(path="/analyze/chunks", audit_name="chunking_analyzed", operation="chunking"),
            AnalysisRoute(path="/analyze/skeleton", audit_name="skeleton_analyzed", operation="skeleton"),
            AnalysisRoute(path="/analyze/reader-plan", audit_name="reader_plan_analyzed", operation="reader_plan"),
            AnalysisRoute(path="/analyze/listening-plan", audit_name="listening_plan_analyzed", operation="listening_plan"),
            AnalysisRoute(path="/analyze/speaking-plan", audit_name="speaking_plan_analyzed", operation="speaking_plan"),
            AnalysisRoute(path="/analyze/rescue-plan", audit_name="rescue_plan_analyzed", operation="rescue_plan"),
            AnalysisRoute(path="/analyze/assessment", audit_name="assessment_analyzed", operation="assessment"),
            AnalysisRoute(path="/analyze/collapse-patterns", audit_name="collapse_patterns_analyzed", operation="collapse_patterns"),
            AnalysisRoute(path="/analyze/analytics-summary", audit_name="analytics_summary_analyzed", operation="analytics_summary"),
            AnalysisRoute(path="/analyze/practice-set", audit_name="practice_set_analyzed", operation="practice_set"),
        )
