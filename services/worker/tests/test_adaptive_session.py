from app.adaptive_session import AdaptiveSessionService
from app.analytics_summary import AnalyticsSummaryService
from app.assessment import AssessmentService
from app.collapse_patterns import CollapsePatternService
from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService
from app.practice_set import PracticeSetService
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService


def test_adaptive_session_builds_summary_and_practice_set() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    assessment_service = AssessmentService()
    collapse_pattern_service = CollapsePatternService(chunking_service=chunking_service)
    analytics_summary_service = AnalyticsSummaryService(
        assessment_service=assessment_service,
        collapse_pattern_service=collapse_pattern_service,
    )
    practice_set_service = PracticeSetService(
        reader_plan_service=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_service=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_service=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_service=RescuePlanService(chunking_service=chunking_service),
        assessment_service=assessment_service,
        collapse_pattern_service=collapse_pattern_service,
    )
    service = AdaptiveSessionService(
        analytics_summary_service=analytics_summary_service,
        practice_set_service=practice_set_service,
    )

    result = service.build(
        "The client approved the design draft, but the delivery schedule is still under review.",
        language="en",
        target_context="meeting",
        self_reported_difficulties=["audio_tracking", "speech_breakdown"],
        fatigue_level="high",
        session_events=[
            {"event_type": "audio_restart", "chunk_order": 1, "seconds": 0.0},
            {"event_type": "audio_pause", "chunk_order": 1, "seconds": 1.2},
            {"event_type": "speed_down", "chunk_order": 2, "seconds": 0.0},
        ],
    )

    assert result.recommended_entry_mode == "listening"
    assert result.analytics_summary.collapse_patterns.likely_mode == "listening"
    assert result.practice_set.detected_weak_mode == "listening"
    assert result.practice_set.suggested_order[0] == "listening"
    assert "Primary focus:" in result.session_plan_note
