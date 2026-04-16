from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Final


_DEFAULT_HOST: Final[str] = "127.0.0.1"
_DEFAULT_PORT: Final[int] = 8090
_DEFAULT_MAX_WORDS: Final[int] = 6
_DEFAULT_MAX_BODY_BYTES: Final[int] = 16_384
_DEFAULT_MAX_TEXT_CHARS: Final[int] = 4_000
_DEFAULT_REQUEST_TIMEOUT_SECONDS: Final[int] = 10


@dataclass(slots=True)
class Settings:
    host: str = _DEFAULT_HOST
    port: int = _DEFAULT_PORT
    max_words_per_chunk: int = _DEFAULT_MAX_WORDS
    require_api_key: bool = True
    api_keys: tuple[str, ...] = ()
    max_body_bytes: int = _DEFAULT_MAX_BODY_BYTES
    max_text_chars: int = _DEFAULT_MAX_TEXT_CHARS
    request_timeout_seconds: int = _DEFAULT_REQUEST_TIMEOUT_SECONDS

    @classmethod
    def load(cls) -> "Settings":
        settings = cls(
            host=os.getenv("WORKER_HOST", _DEFAULT_HOST),
            port=_get_int("WORKER_PORT", _DEFAULT_PORT),
            max_words_per_chunk=_get_int("CHUNKING_MAX_WORDS", _DEFAULT_MAX_WORDS),
            require_api_key=_get_bool("WORKER_REQUIRE_API_KEY", True),
            api_keys=_get_csv("WORKER_API_KEYS"),
            max_body_bytes=_get_int("WORKER_MAX_BODY_BYTES", _DEFAULT_MAX_BODY_BYTES),
            max_text_chars=_get_int("WORKER_MAX_TEXT_CHARS", _DEFAULT_MAX_TEXT_CHARS),
            request_timeout_seconds=_get_int("WORKER_REQUEST_TIMEOUT_SECONDS", _DEFAULT_REQUEST_TIMEOUT_SECONDS),
        )
        settings.validate()
        return settings

    def validate(self) -> None:
        if self.port < 0:
            raise ValueError("WORKER_PORT must be zero or positive")
        if self.max_words_per_chunk <= 0:
            raise ValueError("CHUNKING_MAX_WORDS must be positive")
        if self.max_body_bytes <= 0:
            raise ValueError("WORKER_MAX_BODY_BYTES must be positive")
        if self.max_text_chars <= 0:
            raise ValueError("WORKER_MAX_TEXT_CHARS must be positive")
        if self.request_timeout_seconds <= 0:
            raise ValueError("WORKER_REQUEST_TIMEOUT_SECONDS must be positive")
        if self.require_api_key and not self.api_keys:
            raise ValueError("WORKER_API_KEYS must be set when WORKER_REQUIRE_API_KEY=true")


def _get_int(name: str, fallback: int) -> int:
    value = os.getenv(name)
    if value is None:
        return fallback

    try:
        return int(value)
    except ValueError:
        return fallback


def _get_bool(name: str, fallback: bool) -> bool:
    value = os.getenv(name)
    if value is None:
        return fallback
    return value.strip().lower() in {"1", "true", "yes", "on"}


def _get_csv(name: str) -> tuple[str, ...]:
    value = os.getenv(name, "")
    return tuple(part.strip() for part in value.split(",") if part.strip())
