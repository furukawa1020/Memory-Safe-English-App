from app.assessment import AssessmentService
from app.analysis import AnalysisService, AnalyzeTextInput
from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService
from app.reader_plan import ReaderPlanService
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService
from app.skeleton import SkeletonService


def test_analysis_service_dispatches_chunking() -> None:
    service = AnalysisService(
        chunk_analyzer=ChunkingService(max_words_per_chunk=4),
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        listening_plan_analyzer=ListeningPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        rescue_plan_analyzer=RescuePlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("chunking", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "chunks", None)


def test_analysis_service_dispatches_skeleton() -> None:
    service = AnalysisService(
        chunk_analyzer=ChunkingService(max_words_per_chunk=4),
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        listening_plan_analyzer=ListeningPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        rescue_plan_analyzer=RescuePlanService(chunking_service=ChunkingService(max_words_per_chunk=4)),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("skeleton", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "parts", None)


def test_analysis_service_dispatches_reader_plan() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("reader_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_mode == "progressive"
    assert result.focus_steps


def test_analysis_service_dispatches_listening_plan() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("listening_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_speed
    assert result.pause_points


def test_analysis_service_dispatches_speaking_plan() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("speaking_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface.", language="en"))

    assert result.summary
    assert result.recommended_style == "short-linked-sentences"
    assert result.steps


def test_analysis_service_dispatches_rescue_plan() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=AssessmentService(),
    )

    result = service.analyze("rescue_plan", AnalyzeTextInput(text="In this study, we propose a memory safe interface that reduces overload during reading.", language="en"))

    assert result.summary
    assert result.primary_strategy
    assert result.phrases


def test_analysis_service_dispatches_assessment() -> None:
    chunking_service = ChunkingService(max_words_per_chunk=4)
    service = AnalysisService(
        chunk_analyzer=chunking_service,
        skeleton_analyzer=SkeletonService(),
        reader_plan_analyzer=ReaderPlanService(chunking_service=chunking_service),
        listening_plan_analyzer=ListeningPlanService(chunking_service=chunking_service),
        speaking_plan_analyzer=SpeakingPlanService(chunking_service=chunking_service),
        rescue_plan_analyzer=RescuePlanService(chunking_service=chunking_service),
        assessment_analyzer=AssessmentService(),
    )

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
