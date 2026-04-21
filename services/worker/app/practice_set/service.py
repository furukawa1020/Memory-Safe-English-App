from __future__ import annotations

from dataclasses import dataclass

from app.assessment import AssessmentService
from app.context_profile import resolve_context_profile
from app.listening_plan import ListeningPlanService
from app.models import PracticeSection, PracticeSetResult, PracticeTask, RESPONSE_VERSION
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService


@dataclass(slots=True)
class PracticeSetService:
    reader_plan_service: ReaderPlanService
    listening_plan_service: ListeningPlanService
    speaking_plan_service: SpeakingPlanService
    rescue_plan_service: RescuePlanService
    assessment_service: AssessmentService

    def build(
        self,
        text: str,
        *,
        language: str = "en",
        target_context: str = "general",
        self_reported_difficulties: list[str] | None = None,
        fatigue_level: str = "unknown",
    ) -> PracticeSetResult:
        profile = resolve_context_profile(target_context)
        assessment = self.assessment_service.assess(
            text=text,
            language=language,
            target_context=target_context,
            self_reported_difficulties=self_reported_difficulties,
            fatigue_level=fatigue_level,
        )
        reader_plan = self.reader_plan_service.build(text=text, language=language, target_context=target_context)
        listening_plan = self.listening_plan_service.build(text=text, language=language, target_context=target_context)
        speaking_plan = self.speaking_plan_service.build(text=text, language=language, target_context=target_context)
        rescue_plan = self.rescue_plan_service.build(text=text, language=language, target_context=target_context)

        sections = [
            PracticeSection(
                mode="reading",
                goal=f"Read the main idea first for {profile.label_ja}.",
                tasks=_build_reading_tasks(reader_plan),
            ),
            PracticeSection(
                mode="listening",
                goal="Keep only one checkpoint in memory at a time.",
                tasks=_build_listening_tasks(listening_plan),
            ),
            PracticeSection(
                mode="speaking",
                goal="Say short linked sentences without holding the whole paragraph.",
                tasks=_build_speaking_tasks(speaking_plan),
            ),
            PracticeSection(
                mode="rescue",
                goal="Keep the interaction alive when overload starts.",
                tasks=_build_rescue_tasks(rescue_plan),
            ),
        ]

        return PracticeSetResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            summary=reader_plan.summary or speaking_plan.summary or text[:80],
            suggested_order=_build_suggested_order(assessment),
            sections=sections,
        )


def _build_reading_tasks(reader_plan) -> list[PracticeTask]:
    tasks: list[PracticeTask] = []
    for step in reader_plan.focus_steps[:3]:
        support = " / ".join([*step.support_before[:1], *step.support_after[:1]])
        tasks.append(
            PracticeTask(
                task_id=f"reading-{step.step}",
                mode="reading",
                title=f"Focus chunk {step.step}",
                prompt=f"Read only '{step.text}' first. Confirm the core meaning before opening support.",
                expected_focus=step.guidance_en,
                support=support or step.presentation_hint,
                difficulty=step.overload_risk,
            )
        )
    return tasks


def _build_listening_tasks(listening_plan) -> list[PracticeTask]:
    tasks: list[PracticeTask] = []
    for point in listening_plan.pause_points[:3]:
        tasks.append(
            PracticeTask(
                task_id=f"listening-{point.index}",
                mode="listening",
                title=f"Pause point {point.index}",
                prompt="Listen only to this checkpoint and stop there before moving on.",
                expected_focus=point.cue_en,
                support=point.preview_text,
                difficulty=point.risk_level,
            )
        )
    if not tasks:
        tasks.append(
            PracticeTask(
                task_id="listening-1",
                mode="listening",
                title="Single pass preview",
                prompt="Listen once at the recommended speed and keep only the main point.",
                expected_focus=listening_plan.final_pass_strategy,
                support=f"Recommended speed: {listening_plan.recommended_speed}",
                difficulty="low",
            )
        )
    return tasks


def _build_speaking_tasks(speaking_plan) -> list[PracticeTask]:
    tasks: list[PracticeTask] = []
    if speaking_plan.opener_options:
        tasks.append(
            PracticeTask(
                task_id="speaking-opener",
                mode="speaking",
                title="Opener practice",
                prompt=f"Start with: '{speaking_plan.opener_options[0]}'",
                expected_focus="Open with the summary, not the full sentence.",
                support=", ".join(speaking_plan.bridge_phrases[:2]) or "First, Next,",
                difficulty="low",
            )
        )
    for step in speaking_plan.steps[:3]:
        tasks.append(
            PracticeTask(
                task_id=f"speaking-{step.step}",
                mode="speaking",
                title=f"Speaking step {step.step}",
                prompt=f"Say this as one short unit: '{step.text}'",
                expected_focus=step.delivery_tip_en,
                support=step.purpose,
                difficulty=step.risk_level,
            )
        )
    return tasks


def _build_rescue_tasks(rescue_plan) -> list[PracticeTask]:
    tasks: list[PracticeTask] = []
    for index, phrase in enumerate(rescue_plan.phrases[:3], start=1):
        tasks.append(
            PracticeTask(
                task_id=f"rescue-{index}",
                mode="rescue",
                title=f"Rescue phrase {index}",
                prompt=f"Practice saying: '{phrase.phrase_en}'",
                expected_focus=phrase.use_when,
                support=phrase.phrase_ja,
                difficulty="medium" if phrase.priority <= 2 else "low",
            )
        )
    return tasks


def _build_suggested_order(assessment) -> list[str]:
    scores = {
        "reading": assessment.reading_load_score,
        "listening": assessment.listening_load_score,
        "speaking": assessment.speaking_load_score,
        "rescue": max(assessment.listening_load_score, assessment.speaking_load_score),
    }
    ordered = sorted(scores.items(), key=lambda item: (-item[1], item[0]))
    return [mode for mode, _ in ordered]
