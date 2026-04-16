from __future__ import annotations

import os
from dataclasses import dataclass


@dataclass(slots=True)
class Settings:
    host: str = "127.0.0.1"
    port: int = 8090
    max_words_per_chunk: int = 6

    @classmethod
    def load(cls) -> "Settings":
        return cls(
            host=os.getenv("WORKER_HOST", "127.0.0.1"),
            port=_get_int("WORKER_PORT", 8090),
            max_words_per_chunk=_get_int("CHUNKING_MAX_WORDS", 6),
        )


def _get_int(name: str, fallback: int) -> int:
    value = os.getenv(name)
    if value is None:
        return fallback

    try:
        return int(value)
    except ValueError:
        return fallback
