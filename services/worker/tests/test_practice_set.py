from app.assessment import AssessmentService
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
    assert len(result.sections) == 4
    assert result.sections[0].mode == "reading"
    assert result.sections[0].tasks
    assert result.sections[1].mode == "listening"
    assert result.sections[2].mode == "speaking"
    assert result.sections[3].mode == "rescue"
    assert all(section.tasks for section in result.sections)
