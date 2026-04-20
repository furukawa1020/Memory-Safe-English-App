from app.chunking import ChunkingService
from app.collapse_patterns import CollapsePatternService


def test_collapse_patterns_detects_high_risk_site_from_events() -> None:
    service = CollapsePatternService(chunking_service=ChunkingService(max_words_per_chunk=4))

    result = service.analyze(
        "In this study, we propose a memory safe interface that reduces overload during reading.",
        language="en",
        session_events=[
            {"event_type": "repeat", "chunk_order": 2, "seconds": 0.0},
            {"event_type": "support_open", "chunk_order": 2, "seconds": 0.0},
            {"event_type": "long_pause", "chunk_order": 2, "seconds": 5.0},
        ],
    )

    assert result.dominant_pattern
    assert result.likely_mode == "reading"
    assert result.mode_signals["reading"] >= 3
    assert result.sites
    assert result.sites[0].recommendation
