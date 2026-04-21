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
                why_this_works="It removes the need to hold every modifier before the main claim is stable.",
                tasks=_build_reading_tasks(reader_plan),
            ),
            PracticeSection(
                mode="listening",
                goal="Keep only one checkpoint in memory at a time.",
                why_this_works="It turns continuous audio into short holding windows with explicit stop points.",
                tasks=_build_listening_tasks(listening_plan),
            ),
            PracticeSection(
                mode="speaking",
                goal="Say short linked sentences without holding the whole paragraph.",
                why_this_works="It replaces one fragile long sentence with small units you can complete safely.",
                tasks=_build_speaking_tasks(speaking_plan),
            ),
            PracticeSection(
                mode="rescue",
                goal="Keep the interaction alive when overload starts.",
                why_this_works="It gives a fixed phrase before breakdown so the conversation does not collapse.",
                tasks=_build_rescue_tasks(rescue_plan),
            ),
        ]

        return PracticeSetResult(
            version=RESPONSE_VERSION,
            language=language,
            target_context=target_context,
            summary=reader_plan.summary or speaking_plan.summary or text[:80],
            suggested_order=_build_suggested_order(assessment),
            profile_note=_build_profile_note(assessment),
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
                problem_type="core_lock",
                title=f"Focus chunk {step.step}",
                prompt=f"Read only '{step.text}' first. Confirm the core meaning before opening support.",
                expected_focus=step.guidance_en,
                support=support or step.presentation_hint,
                difficulty=step.overload_risk,
                wm_support="Hide support until the core chunk feels stable.",
                success_check="You can say the main claim in one short Japanese or English phrase.",
            )
        )
        if support:
            tasks.append(
                PracticeTask(
                    task_id=f"reading-support-{step.step}",
                    mode="reading",
                    problem_type="support_attach",
                    title=f"Attach support {step.step}",
                    prompt=f"Now add this support to the main chunk without rereading everything: '{support}'.",
                    expected_focus="Add one support detail while keeping the same core idea active.",
                    support=step.text,
                    difficulty=step.overload_risk,
                    wm_support="Attach only one support detail at a time.",
                    success_check="You can explain how the support connects to the core in one sentence.",
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
                problem_type="pause_recall",
                title=f"Pause point {point.index}",
                prompt="Listen only to this checkpoint and stop there before moving on.",
                expected_focus=point.cue_en,
                support=point.preview_text,
                difficulty=point.risk_level,
                wm_support="You are allowed to forget later audio because the pause protects the current chunk.",
                success_check="You can say the checkpoint meaning before hearing the next chunk.",
            )
        )
        tasks.append(
            PracticeTask(
                task_id=f"listening-check-{point.index}",
                mode="listening",
                problem_type="meaning_hold",
                title=f"Meaning hold {point.index}",
                prompt="After the pause, say only the main point, not every word.",
                expected_focus="Keep the gist only and drop exact wording.",
                support=point.cue_ja,
                difficulty=point.risk_level,
                wm_support="This task rewards gist retention instead of verbatim memory.",
                success_check="You can restate the chunk without replaying it immediately.",
            )
        )
    if not tasks:
        tasks.append(
            PracticeTask(
                task_id="listening-1",
                mode="listening",
                problem_type="single_pass_preview",
                title="Single pass preview",
                prompt="Listen once at the recommended speed and keep only the main point.",
                expected_focus=listening_plan.final_pass_strategy,
                support=f"Recommended speed: {listening_plan.recommended_speed}",
                difficulty="low",
                wm_support="There is only one target: the main point.",
                success_check="You can tell what the sentence was mainly about after one pass.",
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
                problem_type="opener_only",
                title="Opener practice",
                prompt=f"Start with: '{speaking_plan.opener_options[0]}'",
                expected_focus="Open with the summary, not the full sentence.",
                support=", ".join(speaking_plan.bridge_phrases[:2]) or "First, Next,",
                difficulty="low",
                wm_support="You only need the opener, not the whole answer.",
                success_check="You can start speaking within two seconds without building the rest first.",
            )
        )
    for step in speaking_plan.steps[:3]:
        tasks.append(
            PracticeTask(
                task_id=f"speaking-{step.step}",
                mode="speaking",
                problem_type="short_unit",
                title=f"Speaking step {step.step}",
                prompt=f"Say this as one short unit: '{step.text}'",
                expected_focus=step.delivery_tip_en,
                support=step.purpose,
                difficulty=step.risk_level,
                wm_support="Finish one unit before planning the next one.",
                success_check="You can say the step smoothly in one breath.",
            )
        )
    if len(speaking_plan.steps) >= 2:
        tasks.append(
            PracticeTask(
                task_id="speaking-link",
                mode="speaking",
                problem_type="two_step_link",
                title="Link two short steps",
                prompt=f"Say step 1, pause, then add step 2 with a bridge like '{(speaking_plan.bridge_phrases[:1] or ['Next,'])[0]}'",
                expected_focus="Keep the connection simple instead of merging everything into one sentence.",
                support="Pause between short units is allowed.",
                difficulty="medium",
                wm_support="A bridge phrase replaces the need to hold a complex sentence plan.",
                success_check="You can connect two short units without losing the second one.",
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
                problem_type="rescue_phrase",
                title=f"Rescue phrase {index}",
                prompt=f"Practice saying: '{phrase.phrase_en}'",
                expected_focus=phrase.use_when,
                support=phrase.phrase_ja,
                difficulty="medium" if phrase.priority <= 2 else "low",
                wm_support="The phrase is preloaded so you do not need to build a sentence under pressure.",
                success_check="You can say the phrase immediately when overload starts.",
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


def _build_profile_note(assessment) -> str:
    strongest_area = max(
        [
            ("reading", assessment.reading_load_score),
            ("listening", assessment.listening_load_score),
            ("speaking", assessment.speaking_load_score),
        ],
        key=lambda item: item[1],
    )[0]
    return (
        f"Start with {strongest_area} support. "
        f"Use reader mode '{assessment.recommended_reader_mode}', "
        f"listening mode '{assessment.recommended_listening_mode}', "
        f"and speaking mode '{assessment.recommended_speaking_mode}'."
    )
