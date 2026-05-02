from app.assessment import AssessmentService
from app.collapse_patterns import CollapsePatternService
from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService
from app.practice_set import PracticeSetService
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService


def test_practice_set_builds_multi_mode_tasks() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = PracticeSetService(
        reader_plan_service=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_service=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_service=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_service=RescuePlanService(chunking_service=chunking_service),
        assessment_service=AssessmentService(),
        collapse_pattern_service=CollapsePatternService(chunking_service=chunking_service),
    )

    result = service.build(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="research",
        self_reported_difficulties=["sentence_integration", "audio_tracking"],
        fatigue_level="medium",
    )

    assert result.summary
    assert result.suggested_order
    assert result.profile_note
    assert result.detected_weak_mode
    assert result.collapse_summary
    assert result.adaptive_reason
    assert result.profile_note.startswith(f"Start with {result.suggested_order[0]}")
    assert len(result.sections) == 4
    assert result.sections[0].mode == result.suggested_order[0]
    assert result.sections[0].why_this_works
    assert result.sections[0].tasks
    assert result.sections[1].mode == "listening"
    assert result.sections[2].mode == "speaking"
    assert result.sections[3].mode == "rescue"
    assert all(section.tasks for section in result.sections)
    first_task = result.sections[0].tasks[0]
    assert first_task.problem_type
    assert first_task.wm_support
    assert first_task.success_check


def test_practice_set_prioritizes_listening_when_audio_events_dominate() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = PracticeSetService(
        reader_plan_service=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_service=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_service=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_service=RescuePlanService(chunking_service=chunking_service),
        assessment_service=AssessmentService(),
        collapse_pattern_service=CollapsePatternService(chunking_service=chunking_service),
    )

    result = service.build(
        "The client approved the design draft, but the delivery schedule is still under review.",
        language="en",
        target_context="meeting",
        self_reported_difficulties=["audio_tracking", "fast_audio"],
        fatigue_level="high",
        session_events=[
            {"event_type": "audio_restart", "chunk_order": 1, "seconds": 0.0},
            {"event_type": "audio_pause", "chunk_order": 1, "seconds": 1.2},
            {"event_type": "speed_down", "chunk_order": 2, "seconds": 0.0},
        ],
    )

    assert result.detected_weak_mode == "listening"
    assert result.suggested_order[0] == "listening"
    assert result.sections[0].mode == "listening"
    assert "listening" in result.adaptive_reason
    assert "weakest mode" in result.profile_note
