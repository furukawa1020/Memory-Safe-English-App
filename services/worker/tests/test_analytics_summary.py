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
    assert "claim" in result.recommendations[0].title.lower() or "method" in result.recommendations[0].title.lower()


def test_analytics_summary_prioritizes_listening_when_audio_events_dominate() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalyticsSummaryService(
        assessment_service=AssessmentService(),
        collapse_pattern_service=CollapsePatternService(chunking_service=chunking_service),
    )

    result = service.summarize(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="meeting",
        self_reported_difficulties=["audio_tracking", "fast_audio", "sentence_integration"],
        fatigue_level="high",
        session_events=[
            {"event_type": "audio_restart", "chunk_order": 2, "seconds": 0.0},
            {"event_type": "audio_pause", "chunk_order": 2, "seconds": 1.5},
            {"event_type": "speed_down", "chunk_order": 3, "seconds": 0.0},
        ],
    )

    assert result.collapse_patterns.likely_mode == "listening"
    listening_items = [item for item in result.recommendations if item.area == "listening"]
    reading_items = [item for item in result.recommendations if item.area == "reading"]
    assert listening_items
    assert reading_items
    assert listening_items[0].priority < reading_items[0].priority
    assert "current main collapse mode" in listening_items[0].reason
