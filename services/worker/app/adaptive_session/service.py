from __future__ import annotations

from dataclasses import dataclass

from app.analytics_summary import AnalyticsSummaryService
from app.models import AdaptiveSessionResult, RESPONSE_VERSION
from app.practice_set import PracticeSetService


@dataclass(slots=True)
class AdaptiveSessionService:
    analytics_summary_service: AnalyticsSummaryService
    practice_set_service: PracticeSetService

    def build(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
        session_events: list[dict[str, str | int | float]] | None = None,
    ) -> AdaptiveSessionResult:
        analytics_summary = self.analytics_summary_service.summarize(
            text=text,
            language=language,
            target_context=target_context,
            self_reported_difficulties=self_reported_difficulties,
            fatigue_level=fatigue_level,
            session_events=session_events,
        )
        practice_set = self.practice_set_service.build(
            text=text,
            language=language,
            target_context=target_context,
            self_reported_difficulties=self_reported_difficulties,
            fatigue_level=fatigue_level,
            session_events=session_events,
        )

        recommended_entry_mode = (
            practice_set.suggested_order[0]
            if practice_set.suggested_order
            else practice_set.detected_weak_mode
        )
        session_plan_note = _build_session_plan_note(
            recommended_entry_mode=recommended_entry_mode,
            next_focus=analytics_summary.next_focus,
            adaptive_reason=practice_set.adaptive_reason,
        )

        return AdaptiveSessionResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            recommended_entry_mode=recommended_entry_mode,
            session_plan_note=session_plan_note,
            analytics_summary=analytics_summary,
            practice_set=practice_set,
        )


def _build_session_plan_note(
    *,
    recommended_entry_mode: str,
    next_focus: str,
    adaptive_reason: str,
) -> str:
    mode_label = recommended_entry_mode.replace("_", " ")
    return (
        f"Start this session with {mode_label} support. "
        f"Primary focus: {next_focus}. "
        f"Reason: {adaptive_reason}"
    )
