from __future__ import annotations

from dataclasses import dataclass


_DEFAULT_LANGUAGE = "en"


@dataclass(slots=True)
class AnalyzeTextInput:
    text: str
    language: str = _DEFAULT_LANGUAGE

    @classmethod
    def from_payload(cls, payload: object) -> "AnalyzeTextInput":
        if not isinstance(payload, dict):
            raise ValueError("request body must be a JSON object")

        text = payload.get("text", "")
        language = payload.get("language", _DEFAULT_LANGUAGE)

        if not isinstance(text, str):
            raise ValueError("text must be a string")
        if not isinstance(language, str):
            raise ValueError("language must be a string")

        normalized_text = text.strip()
        normalized_language = language.strip().lower() or _DEFAULT_LANGUAGE

        if not normalized_text:
            raise ValueError("text is required")
        if not normalized_language.isascii() or not normalized_language.replace("-", "").isalpha():
            raise ValueError("language must be an ASCII language tag")

        return cls(text=normalized_text, language=normalized_language)
