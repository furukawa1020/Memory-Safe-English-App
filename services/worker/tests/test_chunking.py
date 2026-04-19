from app.chunking import ChunkingService


def test_chunking_splits_into_meaningful_groups() -> None:
    service = ChunkingService(max_words_per_chunk=4)

    result = service.chunk_text("In this study, we propose a memory safe interface for English reading.")

    assert len(result.chunks) >= 2
    assert result.chunks[0].role in {"modifier", "core"}
    assert any(chunk.role == "core" for chunk in result.chunks)


def test_chunking_handles_empty_text() -> None:
    service = ChunkingService()

    result = service.chunk_text("")

    assert result.chunks == []
    assert result.summary == ""


def test_chunking_prefers_soft_split_boundaries() -> None:
    service = ChunkingService(max_words_per_chunk=5)

    result = service.chunk_text("We propose a safer interface for English reading support in class.")

    assert len(result.chunks) >= 2
    assert any(chunk.text.startswith("for English") for chunk in result.chunks)
