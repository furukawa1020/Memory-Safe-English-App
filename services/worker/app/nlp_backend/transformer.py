from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from typing import Any

from app.text_analysis import normalize_text, split_long_segment, summarize_segments


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


@dataclass(slots=True)
class TransformerChunkBackend:
    model_name: str
    task: str = "text2text-generation"
    device: int = -1
    max_new_tokens: int = 256
    _pipeline: Any = field(default=None, init=False, repr=False)

    def chunk(self, text: str, *, language: str, max_words_per_chunk: int) -> list[str]:
        normalized = normalize_text(text)
        if not normalized:
            return []

        generated = self._run_pipeline(_build_chunk_prompt(normalized, language, max_words_per_chunk))
        parsed = _parse_generation_payload(generated)
        segments = parsed.get("segments", [])
        if not isinstance(segments, list):
            return self._fallback_segments(normalized, max_words_per_chunk)

        normalized_segments = [
            normalize_text(str(segment))
            for segment in segments
            if isinstance(segment, str) and normalize_text(segment)
        ]
        if not normalized_segments:
            return self._fallback_segments(normalized, max_words_per_chunk)
        return normalized_segments

    def summarize(self, segments: list[str], *, language: str) -> str:
        if not segments:
            return ""

        generated = self._run_pipeline(_build_summary_prompt(segments, language))
        parsed = _parse_generation_payload(generated)
        summary = parsed.get("summary", "")
        if isinstance(summary, str) and summary.strip():
            return normalize_text(summary)
        return summarize_segments(segments, max_segments=2)

    def _run_pipeline(self, prompt: str) -> str:
        if self._pipeline is None:
            try:
                from transformers import pipeline
            except ImportError as exc:
                raise RuntimeError("transformers backend requested but transformers is not installed") from exc

            self._pipeline = pipeline(task=self.task, model=self.model_name, device=self.device)

        outputs = self._pipeline(prompt, max_new_tokens=self.max_new_tokens, truncation=True)
        if not outputs:
            return ""
        first = outputs[0]
        if isinstance(first, dict):
            if "generated_text" in first:
                return str(first["generated_text"])
            if "summary_text" in first:
                return str(first["summary_text"])
        return str(first)

    @staticmethod
    def _fallback_segments(text: str, max_words_per_chunk: int) -> list[str]:
        words = text.split()
        if len(words) <= max_words_per_chunk:
            return [text]
        return split_long_segment(text, max_words_per_chunk)


def _build_chunk_prompt(text: str, language: str, max_words_per_chunk: int) -> str:
    return (
        "You are an English-learning accessibility assistant. "
        "Split the text into short meaning chunks for a learner with low working memory. "
        f"Language: {language}. Maximum words per chunk: {max_words_per_chunk}. "
        'Return strict JSON like {"segments":["...","..."]}. '
        f"Text: {text}"
    )


def _build_summary_prompt(segments: list[str], language: str) -> str:
    joined = " || ".join(segments)
    return (
        "You are an English-learning accessibility assistant. "
        "Write a very short plain summary for the chunk list. "
        f"Language: {language}. "
        'Return strict JSON like {"summary":"..."}. '
        f"Chunks: {joined}"
    )


def _parse_generation_payload(text: str) -> dict[str, Any]:
    if not text:
        return {}
    match = _JSON_BLOCK.search(text)
    candidate = match.group(0) if match else text
    try:
        payload = json.loads(candidate)
    except json.JSONDecodeError:
        return {}
    if isinstance(payload, dict):
        return payload
    return {}
