from app.chunking import ChunkingService
from app.speaking_plan import SpeakingPlanService


def test_speaking_plan_creates_short_steps_and_rescue_phrases() -> None:
    service = SpeakingPlanService(chunking_service=ChunkingService(max_words_per_chunk=4))

    result = service.build(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        target_context="self_intro",
    )

    assert result.recommended_style == "short-linked-sentences"
    assert result.opener_options
    assert result.opener_options[0].startswith("Let me introduce myself")
    assert "this is the main point" not in result.opener_options[0].lower()
    assert result.steps
    assert result.steps[0].delivery_tip_ja
    assert result.rescue_phrases
