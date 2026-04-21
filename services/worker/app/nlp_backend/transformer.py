from __future__ import annotations

import json
import re
from dataclasses import dataclass, field
from typing import Any

from app.text_analysis import normalize_text, split_long_segment, summarize_segments


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


@dataclass(frozen=True, slots=True)
class TransformerGenerationConfig:
    max_input_tokens: int = 512
    max_new_tokens: int = 256
    num_beams: int = 4
    temperature: float = 0.0


@dataclass(slots=True)
class TransformerChunkBackend:
    model_name: str
    task: str = "text2text-generation"
    device: int = -1
    max_new_tokens: int = 256
    max_input_tokens: int = 512
    num_beams: int = 4
    temperature: float = 0.0
    cache_dir: str = ""
    _tokenizer: Any = field(default=None, init=False, repr=False)
    _model: Any = field(default=None, init=False, repr=False)
    _torch: Any = field(default=None, init=False, repr=False)

    def chunk(self, text: str, *, language: str, max_words_per_chunk: int) -> list[str]:
        normalized = normalize_text(text)
        if not normalized:
            return []

        generated = self._generate_text(_build_chunk_prompt(normalized, language, max_words_per_chunk))
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

        generated = self._generate_text(_build_summary_prompt(segments, language))
        parsed = _parse_generation_payload(generated)
        summary = parsed.get("summary", "")
        if isinstance(summary, str) and summary.strip():
            return normalize_text(summary)
        return summarize_segments(segments, max_segments=2)

    def warmup(self) -> None:
        self._ensure_model_loaded()

    def _generate_text(self, prompt: str) -> str:
        tokenizer, model, torch_module = self._ensure_model_loaded()
        inputs = tokenizer(
            prompt,
            return_tensors="pt",
            truncation=True,
            max_length=self.max_input_tokens,
        )
        if self.device >= 0:
            inputs = {key: value.to(model.device) for key, value in inputs.items()}

        generation_kwargs = {
            "max_new_tokens": self.max_new_tokens,
            "num_beams": self.num_beams,
            "do_sample": self.temperature > 0,
            "temperature": self.temperature if self.temperature > 0 else None,
        }
        generation_kwargs = {key: value for key, value in generation_kwargs.items() if value is not None}

        with torch_module.no_grad():
            output_ids = model.generate(**inputs, **generation_kwargs)
        return tokenizer.decode(output_ids[0], skip_special_tokens=True)

    def _ensure_model_loaded(self) -> tuple[Any, Any, Any]:
        if self._tokenizer is not None and self._model is not None and self._torch is not None:
            return self._tokenizer, self._model, self._torch

        try:
            import torch
            from transformers import AutoModelForSeq2SeqLM, AutoTokenizer
        except ImportError as exc:
            raise RuntimeError("transformers backend requested but transformer dependencies are not installed") from exc

        model_kwargs = {}
        if self.cache_dir:
            model_kwargs["cache_dir"] = self.cache_dir

        tokenizer = AutoTokenizer.from_pretrained(self.model_name, **model_kwargs)
        model = AutoModelForSeq2SeqLM.from_pretrained(self.model_name, **model_kwargs)
        if self.device >= 0:
            model = model.to(f"cuda:{self.device}")
        model.eval()

        self._tokenizer = tokenizer
        self._model = model
        self._torch = torch
        return tokenizer, model, torch

    @staticmethod
    def _fallback_segments(text: str, max_words_per_chunk: int) -> list[str]:
        words = text.split()
        if len(words) <= max_words_per_chunk:
            return [text]
        return split_long_segment(text, max_words_per_chunk)


def _build_chunk_prompt(text: str, language: str, max_words_per_chunk: int) -> str:
    return (
        "You are an English-learning accessibility assistant for people with low working memory.\n"
        "Split the text into short meaning chunks.\n"
        f"Language: {language}\n"
        f"Maximum words per chunk: {max_words_per_chunk}\n"
        "Rules:\n"
        "- keep each segment short and meaningful\n"
        "- keep the original word order\n"
        "- do not invent content\n"
        '- return strict JSON only: {"segments":["...","..."]}\n'
        f"Text: {text}"
    )


def _build_summary_prompt(segments: list[str], language: str) -> str:
    joined = " || ".join(segments)
    return (
        "You are an English-learning accessibility assistant for people with low working memory.\n"
        "Write one very short summary that preserves the main meaning.\n"
        f"Language: {language}\n"
        '- return strict JSON only: {"summary":"..."}\n'
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
