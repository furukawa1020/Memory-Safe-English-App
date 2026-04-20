from app.analytics_summary import AnalyticsSummaryService
from app.assessment import AssessmentService
from app.chunking import ChunkingService
from app.collapse_patterns import CollapsePatternService


def test_analytics_summary_returns_next_focus_and_recommendations() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalyticsSummaryService(
        assessment_service=AssessmentService(),
        collapse_pattern_service=CollapsePatternService(chunking_service=chunking_service),
    )

    result = service.summarize(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="research",
        self_reported_difficulties=["sentence_integration", "audio_tracking"],
        fatigue_level="high",
        session_events=[{"event_type": "repeat", "chunk_order": 2, "seconds": 4.0}],
    )

    assert result.next_focus
    assert result.assessment.recommended_reader_mode
    assert result.collapse_patterns.dominant_pattern
    assert result.recommendations
