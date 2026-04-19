from app.analysis import AnalysisService, AnalyzeTextInput
from app.chunking import ChunkingService
from app.skeleton import SkeletonService


def test_analysis_service_dispatches_chunking() -> None:
    service = AnalysisService(
        chunk_analyzer=ChunkingService(max_words_per_chunk=4),
        skeleton_analyzer=SkeletonService(),
    )

    result = service.analyze("chunking", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "chunks", None)


def test_analysis_service_dispatches_skeleton() -> None:
    service = AnalysisService(
        chunk_analyzer=ChunkingService(max_words_per_chunk=4),
        skeleton_analyzer=SkeletonService(),
    )

    result = service.analyze("skeleton", AnalyzeTextInput(text="We propose a memory safe interface.", language="en"))

    assert result.summary
    assert getattr(result, "parts", None)
