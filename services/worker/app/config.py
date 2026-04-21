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
_DEFAULT_REQUIRE_REQUEST_SIGNATURE: Final[bool] = True
_DEFAULT_SIGNATURE_MAX_AGE_SECONDS: Final[int] = 300
_DEFAULT_RATE_LIMIT_MAX_REQUESTS: Final[int] = 30
_DEFAULT_RATE_LIMIT_WINDOW_SECONDS: Final[int] = 60
_DEFAULT_NLP_BACKEND: Final[str] = "heuristic"
_DEFAULT_TRANSFORMER_TASK: Final[str] = "text2text-generation"
_DEFAULT_TRANSFORMER_DEVICE: Final[int] = -1
_DEFAULT_TRANSFORMER_MAX_NEW_TOKENS: Final[int] = 256
_DEFAULT_TRANSFORMER_MAX_INPUT_TOKENS: Final[int] = 512
_DEFAULT_TRANSFORMER_NUM_BEAMS: Final[int] = 4
_DEFAULT_TRANSFORMER_TEMPERATURE: Final[float] = 0.0


@dataclass(slots=True)
class Settings:
    host: str = _DEFAULT_HOST
    port: int = _DEFAULT_PORT
    max_words_per_chunk: int = _DEFAULT_MAX_WORDS
    require_api_key: bool = True
    api_keys: tuple[str, ...] = ()
    require_request_signature: bool = _DEFAULT_REQUIRE_REQUEST_SIGNATURE
    signature_keys: tuple[str, ...] = ()
    signature_max_age_seconds: int = _DEFAULT_SIGNATURE_MAX_AGE_SECONDS
    max_body_bytes: int = _DEFAULT_MAX_BODY_BYTES
    max_text_chars: int = _DEFAULT_MAX_TEXT_CHARS
    request_timeout_seconds: int = _DEFAULT_REQUEST_TIMEOUT_SECONDS
    rate_limit_max_requests: int = _DEFAULT_RATE_LIMIT_MAX_REQUESTS
    rate_limit_window_seconds: int = _DEFAULT_RATE_LIMIT_WINDOW_SECONDS
    nlp_backend: str = _DEFAULT_NLP_BACKEND
    transformer_model_name: str = ""
    transformer_task: str = _DEFAULT_TRANSFORMER_TASK
    transformer_device: int = _DEFAULT_TRANSFORMER_DEVICE
    transformer_max_new_tokens: int = _DEFAULT_TRANSFORMER_MAX_NEW_TOKENS
    transformer_max_input_tokens: int = _DEFAULT_TRANSFORMER_MAX_INPUT_TOKENS
    transformer_num_beams: int = _DEFAULT_TRANSFORMER_NUM_BEAMS
    transformer_temperature: float = _DEFAULT_TRANSFORMER_TEMPERATURE
    transformer_cache_dir: str = ""

    @classmethod
    def load(cls) -> "Settings":
        settings = cls(
            host=os.getenv("WORKER_HOST", _DEFAULT_HOST),
            port=_get_int("WORKER_PORT", _DEFAULT_PORT),
            max_words_per_chunk=_get_int("CHUNKING_MAX_WORDS", _DEFAULT_MAX_WORDS),
            require_api_key=_get_bool("WORKER_REQUIRE_API_KEY", True),
            api_keys=_get_csv("WORKER_API_KEYS"),
            require_request_signature=_get_bool("WORKER_REQUIRE_REQUEST_SIGNATURE", _DEFAULT_REQUIRE_REQUEST_SIGNATURE),
            signature_keys=_get_csv("WORKER_SIGNATURE_KEYS"),
            signature_max_age_seconds=_get_int("WORKER_SIGNATURE_MAX_AGE_SECONDS", _DEFAULT_SIGNATURE_MAX_AGE_SECONDS),
            max_body_bytes=_get_int("WORKER_MAX_BODY_BYTES", _DEFAULT_MAX_BODY_BYTES),
            max_text_chars=_get_int("WORKER_MAX_TEXT_CHARS", _DEFAULT_MAX_TEXT_CHARS),
            request_timeout_seconds=_get_int("WORKER_REQUEST_TIMEOUT_SECONDS", _DEFAULT_REQUEST_TIMEOUT_SECONDS),
            rate_limit_max_requests=_get_int("WORKER_RATE_LIMIT_MAX_REQUESTS", _DEFAULT_RATE_LIMIT_MAX_REQUESTS),
            rate_limit_window_seconds=_get_int("WORKER_RATE_LIMIT_WINDOW_SECONDS", _DEFAULT_RATE_LIMIT_WINDOW_SECONDS),
            nlp_backend=os.getenv("WORKER_NLP_BACKEND", _DEFAULT_NLP_BACKEND).strip().lower(),
            transformer_model_name=os.getenv("WORKER_TRANSFORMER_MODEL", "").strip(),
            transformer_task=os.getenv("WORKER_TRANSFORMER_TASK", _DEFAULT_TRANSFORMER_TASK).strip().lower(),
            transformer_device=_get_int("WORKER_TRANSFORMER_DEVICE", _DEFAULT_TRANSFORMER_DEVICE),
            transformer_max_new_tokens=_get_int("WORKER_TRANSFORMER_MAX_NEW_TOKENS", _DEFAULT_TRANSFORMER_MAX_NEW_TOKENS),
            transformer_max_input_tokens=_get_int("WORKER_TRANSFORMER_MAX_INPUT_TOKENS", _DEFAULT_TRANSFORMER_MAX_INPUT_TOKENS),
            transformer_num_beams=_get_int("WORKER_TRANSFORMER_NUM_BEAMS", _DEFAULT_TRANSFORMER_NUM_BEAMS),
            transformer_temperature=_get_float("WORKER_TRANSFORMER_TEMPERATURE", _DEFAULT_TRANSFORMER_TEMPERATURE),
            transformer_cache_dir=os.getenv("WORKER_TRANSFORMER_CACHE_DIR", "").strip(),
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
        if self.require_request_signature and not self.signature_keys:
            raise ValueError("WORKER_SIGNATURE_KEYS must be set when WORKER_REQUIRE_REQUEST_SIGNATURE=true")
        if self.signature_max_age_seconds <= 0:
            raise ValueError("WORKER_SIGNATURE_MAX_AGE_SECONDS must be positive")
        if self.rate_limit_max_requests <= 0:
            raise ValueError("WORKER_RATE_LIMIT_MAX_REQUESTS must be positive")
        if self.rate_limit_window_seconds <= 0:
            raise ValueError("WORKER_RATE_LIMIT_WINDOW_SECONDS must be positive")
        if self.nlp_backend not in {"heuristic", "transformer"}:
            raise ValueError("WORKER_NLP_BACKEND must be heuristic or transformer")
        if self.nlp_backend == "transformer" and not self.transformer_model_name:
            raise ValueError("WORKER_TRANSFORMER_MODEL must be set when WORKER_NLP_BACKEND=transformer")
        if self.transformer_max_new_tokens <= 0:
            raise ValueError("WORKER_TRANSFORMER_MAX_NEW_TOKENS must be positive")
        if self.transformer_max_input_tokens <= 0:
            raise ValueError("WORKER_TRANSFORMER_MAX_INPUT_TOKENS must be positive")
        if self.transformer_num_beams <= 0:
            raise ValueError("WORKER_TRANSFORMER_NUM_BEAMS must be positive")
        if self.transformer_temperature < 0:
            raise ValueError("WORKER_TRANSFORMER_TEMPERATURE must be zero or positive")


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


def _get_float(name: str, fallback: float) -> float:
    value = os.getenv(name)
    if value is None:
        return fallback

    try:
        return float(value)
    except ValueError:
        return fallback
