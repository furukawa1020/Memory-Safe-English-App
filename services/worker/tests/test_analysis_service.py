from app.analytics_summary import AnalyticsSummaryService
from app.assessment import AssessmentService
from app.analysis import AnalysisService, AnalyzeTextInput
from app.collapse_patterns import CollapsePatternService
from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService
from app.practice_set import PracticeSetService
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService
from app.skeleton import SkeletonService


def _build_analysis_service() -> AnalysisService:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    assessment_service = AssessmentService()
    return AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=assessment_service,
        collapse_pattern_analyzer=CollapsePatternService(chunking_service=chunking_service),
        analytics_summary_analyzer=AnalyticsSummaryService(
            assessment_service=assessment_service,
            collapse_pattern_service=CollapsePatternService(chunking_service=chunking_service),
        ),
        practice_set_analyzer=PracticeSetService(
            reader_plan_service=ReaderPlanService(chunking_service=chunking_service),
            listening_plan_service=ListeningPlanService(chunking_service=chunking_service),
            speaking_plan_service=SpeakingPlanService(chunking_service=chunking_service),
            rescue_plan_service=RescuePlanService(chunking_service=chunking_service),
            assessment_service=assessment_service,
        ),
    )


def test_analysis_service_dispatches_chunking() -> None:
    service = _build_analysis_service()

    result = service.analyze("chunking", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "chunks", None)


def test_analysis_service_dispatches_skeleton() -> None:
    service = _build_analysis_service()

    result = service.analyze("skeleton", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "parts", None)


def test_analysis_service_dispatches_reader_plan() -> None:
    service = _build_analysis_service()

    result = service.analyze("reader_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_mode == "progressive"
    assert result.focus_steps


def test_analysis_service_dispatches_listening_plan() -> None:
    service = _build_analysis_service()

    result = service.analyze("listening_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_speed
    assert result.pause_points


def test_analysis_service_dispatches_speaking_plan() -> None:
    service = _build_analysis_service()

    result = service.analyze("speaking_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_style == "short-linked-sentences"
    assert result.steps


def test_analysis_service_dispatches_rescue_plan() -> None:
    service = _build_analysis_service()

    result = service.analyze("rescue_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface that reduces overload during reading.", language="en"))

    assert result.summary
    assert result.primary_strategy
    assert result.phrases


def test_analysis_service_dispatches_assessment() -> None:
    service = _build_analysis_service()

    result = service.analyze(
        "assessment",
        AnalyzeTextInput(
            text="In this study, we propose a memory safe interface.",
            language="en",
            target_context="research",
            self_reported_difficulties=["sentence_integration", "audio_tracking"],
            fatigue_level="medium",
        ),
    )

    assert result.profile_label
    assert result.recommended_reader_mode
    assert result.recommended_pause_frequency


def test_analysis_service_dispatches_collapse_patterns() -> None:
    service = _build_analysis_service()

    result = service.analyze(
        "collapse_patterns",
        AnalyzeTextInput(
            text="In this study, we propose a memory safe interface that reduces overload during reading.",
            language="en",
            session_events=[
                {"event_type": "repeat", "chunk_order": 2, "seconds": 0.0},
                {"event_type": "long_pause", "chunk_order": 2, "seconds": 4.2},
            ],
        ),
    )

    assert result.dominant_pattern
    assert result.sites


def test_analysis_service_dispatches_analytics_summary() -> None:
    service = _build_analysis_service()

    result = service.analyze(
        "analytics_summary",
        AnalyzeTextInput(
            text="In this study, we propose a memory safe interface that reduces overload during reading.",
            language="en",
            target_context="research",
            self_reported_difficulties=["sentence_integration"],
            fatigue_level="medium",
            session_events=[{"event_type": "repeat", "chunk_order": 2, "seconds": 3.5}],
        ),
    )

    assert result.next_focus
    assert result.recommendations


def test_analysis_service_dispatches_practice_set() -> None:
    service = _build_analysis_service()

    result = service.analyze(
        "practice_set",
        AnalyzeTextInput(
            text="In this study, we propose a memory safe interface that reduces overload during reading.",
            language="en",
            target_context="research",
            self_reported_difficulties=["sentence_integration", "audio_tracking"],
            fatigue_level="medium",
        ),
    )

    assert result.summary
    assert result.suggested_order
    assert result.sections
