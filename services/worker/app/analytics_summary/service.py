from __future__ import annotations

from dataclasses import dataclass

from app.assessment import AssessmentService
from app.collapse_patterns import CollapsePatternService
from app.models import AnalyticsSummaryResult, PracticeRecommendation, RESPONSE_VERSION


@dataclass(slots=True)
class AnalyticsSummaryService:
    assessment_service: AssessmentService
    collapse_pattern_service: CollapsePatternService

    def summarize(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> AnalyticsSummaryResult:
        assessment = self.assessment_service.assess(
            text,
            language=language,
            target_context=target_context,
            self_reported_difficulties=self_reported_difficulties,
            fatigue_level=fatigue_level,
        )
        collapse_patterns = self.collapse_pattern_service.analyze(
            text,
            language=language,
            session_events=session_events,
        )

        recommendations = _build_recommendations(assessment, collapse_patterns)
        next_focus = recommendations[0].title if recommendations else "Keep the current mode and gather more session data."

        return AnalyticsSummaryResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            next_focus=next_focus,
            assessment=assessment,
            collapse_patterns=collapse_patterns,
            recommendations=recommendations,
        )


def _build_recommendations(assessment, collapse_patterns) -> list[PracticeRecommendation]:
    recommendations: list[PracticeRecommendation] = []

    if collapse_patterns.sites:
        recommendations.append(
            PracticeRecommendation(
                area="collapse_pattern",
                title="Review the highest-risk collapse site",
                reason=collapse_patterns.sites[0].recommendation,
                priority=1,
            )
        )
    if assessment.reading_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="reading",
                title="Start with assisted chunk reading",
                reason="Reading load is high enough that support detail should stay collapsed at first.",
                priority=2,
            )
        )
    if assessment.listening_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="listening",
                title="Use chunk-pause listening first",
                reason="Listening load is high enough that frequent pause points will help retain the main idea.",
                priority=3,
            )
        )
    if assessment.speaking_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="speaking",
                title="Switch to template short steps",
                reason="Speaking load is high enough that short linked sentences are safer than longer free speech.",
                priority=4,
            )
        )

    if not recommendations:
        recommendations.append(
            PracticeRecommendation(
                area="stability",
                title="Keep the current mode and gather more usage data",
                reason="No strong overload signal was detected yet.",
                priority=5,
            )
        )

    return sorted(recommendations, key=lambda item: item.priority)
