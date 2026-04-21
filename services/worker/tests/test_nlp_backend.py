from app.chunking import ChunkingService
from app.config import Settings


class FakeChunkBackend:
    def chunk(self, text: str, *, language: str, max_words_per_chunk: int) -> list[str]:
        del text, language, max_words_per_chunk
        return ["We propose", "a safer interface"]

    def summarize(self, segments: list[str], *, language: str) -> str:
        del segments, language
        return "proposal summary"


def test_chunking_service_uses_injected_backend() -> None:
    service = ChunkingService(max_words_per_chunk=4, backend=FakeChunkBackend())

    result = service.chunk_text("We propose a safer interface.", language="en")

    assert [chunk.text for chunk in result.chunks] == ["We propose", "a safer interface"]
    assert result.summary == "proposal summary"


def test_settings_require_transformer_model_when_backend_is_transformer() -> None:
    settings = Settings(
        api_keys=("test-key",),
        signature_keys=("sig-key",),
        nlp_backend="transformer",
    )

    try:
        settings.validate()
    except ValueError as exc:
        assert "WORKER_TRANSFORMER_MODEL" in str(exc)
    else:
        raise AssertionError("expected transformer config validation to fail without model name")


def test_settings_accept_transformer_backend_when_model_is_set() -> None:
    settings = Settings(
        api_keys=("test-key",),
        signature_keys=("sig-key",),
        nlp_backend="transformer",
        transformer_model_name="example/model",
    )

    settings.validate()
