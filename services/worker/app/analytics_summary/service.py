from __future__ import annotations

from dataclasses import dataclass

from app.assessment import AssessmentService
from app.collapse_patterns import CollapsePatternService
from app.context_profile import resolve_context_profile
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
        profile = resolve_context_profile(target_context)
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

        recommendations = _build_recommendations(profile.key, assessment, collapse_patterns)
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


def _build_recommendations(context_key: str, assessment, collapse_patterns) -> list[PracticeRecommendation]:
    recommendations: list[PracticeRecommendation] = []

    if collapse_patterns.sites:
        primary_site = collapse_patterns.sites[0]
        recommendations.append(
            PracticeRecommendation(
                area="collapse_pattern",
                title=_collapse_title(context_key, primary_site.role),
                reason=f"{primary_site.recommendation}. Trigger: {', '.join(primary_site.reasons)}.",
                priority=1,
            )
        )
    if assessment.reading_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="reading",
                title=_reading_title(context_key),
                reason=_reading_reason(context_key),
                priority=2,
            )
        )
    if assessment.listening_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="listening",
                title=_listening_title(context_key),
                reason=_listening_reason(context_key),
                priority=3,
            )
        )
    if assessment.speaking_load_score >= 4:
        recommendations.append(
            PracticeRecommendation(
                area="speaking",
                title=_speaking_title(context_key),
                reason=_speaking_reason(context_key),
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


def _collapse_title(context_key: str, role: str) -> str:
    if context_key == "research":
        return "Review the claim or method chunk that caused the largest collapse"
    if context_key == "meeting":
        return "Review the decision or action-item chunk that caused the largest collapse"
    if context_key == "self_intro":
        return "Review the self-introduction chunk that caused the largest collapse"
    if role == "modifier":
        return "Review the support detail that caused the largest collapse"
    return "Review the highest-risk collapse site"


def _reading_title(context_key: str) -> str:
    if context_key == "research":
        return "Start with assisted reading for claim and method chunks"
    if context_key == "meeting":
        return "Start with assisted reading for decision and action chunks"
    if context_key == "self_intro":
        return "Start with assisted reading for role and personal-summary chunks"
    return "Start with assisted chunk reading"


def _reading_reason(context_key: str) -> str:
    if context_key == "research":
        return "Research sentences often hide the main claim inside dense support, so keep detail collapsed until the core claim is stable."
    if context_key == "meeting":
        return "Meeting language is easier to keep when decision and action chunks stay visible while extra explanation stays collapsed."
    if context_key == "self_intro":
        return "Self-introduction flows are safer when role and main identity chunks stay visible before extra detail appears."
    return "Reading load is high enough that support detail should stay collapsed at first."


def _listening_title(context_key: str) -> str:
    if context_key == "research":
        return "Use checkpoint listening around claim, method, and result"
    if context_key == "meeting":
        return "Use checkpoint listening around decisions and next actions"
    if context_key == "daily":
        return "Use short pause listening around everyday meaning units"
    return "Use chunk-pause listening first"


def _listening_reason(context_key: str) -> str:
    if context_key == "research":
        return "Frequent pauses after claim or result units will reduce the need to retain long academic sentences."
    if context_key == "meeting":
        return "Frequent pauses after decisions or action items will reduce overload during fast meeting turns."
    if context_key == "daily":
        return "Short pauses help keep only the everyday meaning without carrying extra detail forward."
    return "Listening load is high enough that frequent pause points will help retain the main idea."


def _speaking_title(context_key: str) -> str:
    if context_key == "research":
        return "Switch to short research explanation steps"
    if context_key == "meeting":
        return "Switch to short update-and-action speaking steps"
    if context_key == "self_intro":
        return "Switch to short self-introduction steps"
    return "Switch to template short steps"


def _speaking_reason(context_key: str) -> str:
    if context_key == "research":
        return "Short linked steps are safer than long academic explanations when claim, method, and result must stay stable."
    if context_key == "meeting":
        return "Short linked steps are safer than long updates when decisions and next actions need to stay stable."
    if context_key == "self_intro":
        return "Short linked steps are safer than long personal explanations when role, background, and goal must stay stable."
    return "Speaking load is high enough that short linked sentences are safer than longer free speech."
