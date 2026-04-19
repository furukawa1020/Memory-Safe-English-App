from app.chunking import ChunkingService
from app.reader_plan import ReaderPlanService


def test_reader_plan_prioritizes_core_chunks_and_collapses_support() -> None:
    service = ReaderPlanService(chunking_service=ChunkingService(max_words_per_chunk=4))

    result = service.build("In this study, we propose a memory safe interface that reduces overload during reading.", language="en")

    assert result.recommended_mode == "progressive"
    assert result.display_strategy
    assert result.focus_steps
    assert result.focus_steps[0].support_density
    assert result.focus_steps[0].overload_risk in {"low", "medium", "high"}
    assert result.focus_steps[0].presentation_hint
    assert result.focus_steps[0].guidance_ja
    assert any(chunk.role != "core" for chunk in result.collapsed_chunks)
    assert isinstance(result.hotspots, list)
