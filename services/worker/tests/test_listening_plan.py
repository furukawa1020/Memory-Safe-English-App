from app.chunking import ChunkingService
from app.listening_plan import ListeningPlanService


def test_listening_plan_creates_pause_points_and_speed_hint() -> None:
    service = ListeningPlanService(chunking_service=ChunkingService(max_words_per_chunk=4))

    result = service.build(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
    )

    assert result.recommended_speed in {"0.80x", "0.90x", "1.00x"}
    assert result.pause_points
    assert result.pause_points[0].cue_ja
    assert result.final_pass_strategy
