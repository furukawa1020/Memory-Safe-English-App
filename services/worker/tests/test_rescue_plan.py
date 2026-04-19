from app.chunking import ChunkingService
from app.rescue_plan import RescuePlanService


def test_rescue_plan_returns_prioritized_rescue_phrases() -> None:
    service = RescuePlanService(chunking_service=ChunkingService(max_words_per_chunk=4))

    result = service.build(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="meeting",
    )

    assert result.overload_level in {"low", "medium", "high"}
    assert result.primary_strategy
    assert result.phrases
    assert result.phrases[0].category == "shorter"
    assert result.phrases[0].phrase_ja
