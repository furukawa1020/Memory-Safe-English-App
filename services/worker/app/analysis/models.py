from __future__ import annotations

from dataclasses import dataclass


_DEFAULT_LANGUAGE = "en"
_DEFAULT_TARGET_CONTEXT = "general"


@dataclass(slots=True)
class AnalyzeTextInput:
    text: str
    language: str = _DEFAULT_LANGUAGE
    target_context: str = _DEFAULT_TARGET_CONTEXT

    @classmethod
    def from_payload(cls, payload: object) -> "AnalyzeTextInput":
        if not isinstance(payload, dict):
            raise ValueError("request body must be a JSON object")

        text = payload.get("text", "")
        language = payload.get("language", _DEFAULT_LANGUAGE)
        target_context = payload.get("target_context", _DEFAULT_TARGET_CONTEXT)

        if not isinstance(text, str):
            raise ValueError("text must be a string")
        if not isinstance(language, str):
            raise ValueError("language must be a string")
        if not isinstance(target_context, str):
            raise ValueError("target_context must be a string")

        normalized_text = text.strip()
        normalized_language = language.strip().lower() or _DEFAULT_LANGUAGE
        normalized_target_context = target_context.strip().lower() or _DEFAULT_TARGET_CONTEXT

        if not normalized_text:
            raise ValueError("text is required")
        if not normalized_language.isascii() or not normalized_language.replace("-", "").isalpha():
            raise ValueError("language must be an ASCII language tag")
        if not normalized_target_context.isascii() or "_" in normalized_target_context and not normalized_target_context.replace("_", "").isalpha():
            raise ValueError("target_context must be an ASCII context key")

        return cls(
            text=normalized_text,
            language=normalized_language,
            target_context=normalized_target_context,
        )
