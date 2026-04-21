from __future__ import annotations

from typing import Protocol


class ChunkBackend(Protocol):
    def chunk(self, text: str, *, language: str, max_words_per_chunk: int) -> list[str]: ...

    def summarize(self, segments: list[str], *, language: str) -> str: ...
