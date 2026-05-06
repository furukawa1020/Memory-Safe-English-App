from __future__ import annotations

from dataclasses import dataclass

from app.analytics_summary import AnalyticsSummaryService
from app.models import AdaptiveSessionResult, AdaptiveSessionStep, RESPONSE_VERSION
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
        startup_sequence = _build_startup_sequence(
            recommended_entry_mode=recommended_entry_mode,
            practice_set=practice_set,
        )

        return AdaptiveSessionResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            recommended_entry_mode=recommended_entry_mode,
            session_plan_note=session_plan_note,
            startup_sequence=startup_sequence,
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


def _build_startup_sequence(
    *,
    recommended_entry_mode: str,
    practice_set,
) -> list[AdaptiveSessionStep]:
    sections_by_mode = {section.mode: section for section in practice_set.sections}
    ordered_modes = [recommended_entry_mode]
    ordered_modes.extend(
        mode for mode in practice_set.suggested_order if mode not in ordered_modes
    )

    steps: list[AdaptiveSessionStep] = []
    for mode in ordered_modes:
        section = sections_by_mode.get(mode)
        if section is None:
            continue
        first_task = section.tasks[0] if section.tasks else None
        if first_task is None:
            continue

        steps.append(
            AdaptiveSessionStep(
                step=len(steps) + 1,
                mode=mode,
                title=section.goal,
                instruction=first_task.prompt,
                reason=section.why_this_works,
                estimated_minutes=_estimate_minutes(mode),
            )
        )
        if len(steps) >= 3:
            break

    return steps


def _estimate_minutes(mode: str) -> int:
    if mode == "reading":
        return 4
    if mode == "listening":
        return 3
    if mode == "speaking":
        return 4
    return 2
