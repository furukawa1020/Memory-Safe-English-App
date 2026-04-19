from __future__ import annotations

from dataclasses import dataclass


_DEFAULT_LANGUAGE = "en"
_DEFAULT_TARGET_CONTEXT = "general"
_DEFAULT_FATIGUE_LEVEL = "unknown"
_VALID_FATIGUE_LEVELS = {"unknown", "low", "medium", "high"}


@dataclass(slots=True)
class AnalyzeTextInput:
    text: str
    language: str = _DEFAULT_LANGUAGE
    target_context: str = _DEFAULT_TARGET_CONTEXT
    self_reported_difficulties: list[str] | None = None
    fatigue_level: str = _DEFAULT_FATIGUE_LEVEL
    session_events: list[dict[str, str | int | float]] | None = None

    @classmethod
    def from_payload(cls, payload: object) -> "AnalyzeTextInput":
        if not isinstance(payload, dict):
            raise ValueError("request body must be a JSON object")

        text = payload.get("text", "")
        language = payload.get("language", _DEFAULT_LANGUAGE)
        target_context = payload.get("target_context", _DEFAULT_TARGET_CONTEXT)
        self_reported_difficulties = payload.get("self_reported_difficulties", [])
        fatigue_level = payload.get("fatigue_level", _DEFAULT_FATIGUE_LEVEL)
        session_events = payload.get("session_events", [])

        if not isinstance(text, str):
            raise ValueError("text must be a string")
        if not isinstance(language, str):
            raise ValueError("language must be a string")
        if not isinstance(target_context, str):
            raise ValueError("target_context must be a string")
        if not isinstance(self_reported_difficulties, list):
            raise ValueError("self_reported_difficulties must be a list of strings")
        if not isinstance(fatigue_level, str):
            raise ValueError("fatigue_level must be a string")
        if not isinstance(session_events, list):
            raise ValueError("session_events must be a list of event objects")

        normalized_text = text.strip()
        normalized_language = language.strip().lower() or _DEFAULT_LANGUAGE
        normalized_target_context = target_context.strip().lower() or _DEFAULT_TARGET_CONTEXT
        normalized_difficulties = []
        for item in self_reported_difficulties:
            if not isinstance(item, str):
                raise ValueError("self_reported_difficulties must be a list of strings")
            normalized_item = item.strip().lower()
            if normalized_item:
                normalized_difficulties.append(normalized_item)
        normalized_fatigue_level = fatigue_level.strip().lower() or _DEFAULT_FATIGUE_LEVEL
        normalized_events: list[dict[str, str | int | float]] = []
        for item in session_events:
            if not isinstance(item, dict):
                raise ValueError("session_events must be a list of event objects")
            event_type = item.get("event_type", "")
            chunk_order = item.get("chunk_order", 0)
            seconds = item.get("seconds", 0)
            if not isinstance(event_type, str):
                raise ValueError("session_events.event_type must be a string")
            if not isinstance(chunk_order, int):
                raise ValueError("session_events.chunk_order must be an integer")
            if not isinstance(seconds, (int, float)):
                raise ValueError("session_events.seconds must be numeric")
            normalized_events.append(
                {
                    "event_type": event_type.strip().lower(),
                    "chunk_order": chunk_order,
                    "seconds": float(seconds),
                }
            )

        if not normalized_text:
            raise ValueError("text is required")
        if not normalized_language.isascii() or not normalized_language.replace("-", "").isalpha():
            raise ValueError("language must be an ASCII language tag")
        context_key = normalized_target_context.replace("_", "")
        if not normalized_target_context.isascii() or not context_key.isalpha():
            raise ValueError("target_context must be an ASCII context key")
        if normalized_fatigue_level not in _VALID_FATIGUE_LEVELS:
            raise ValueError("fatigue_level must be one of unknown, low, medium, high")

        return cls(
            text=normalized_text,
            language=normalized_language,
            target_context=normalized_target_context,
            self_reported_difficulties=normalized_difficulties,
            fatigue_level=normalized_fatigue_level,
            session_events=normalized_events,
        )
