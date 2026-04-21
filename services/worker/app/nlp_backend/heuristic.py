from __future__ import annotations

from dataclasses import dataclass

from app.text_analysis import normalize_text, segment_text, split_long_segment, summarize_segments


@dataclass(slots=True)
class HeuristicChunkBackend:
    def chunk(self, text: str, *, language: str, max_words_per_chunk: int) -> list[str]:
        del language
        normalized = normalize_text(text)
        if not normalized:
            return []

        segments = segment_text(normalized)
        if not segments:
            segments = [normalized]

        refined: list[str] = []
        for segment in segments:
            refined.extend(split_long_segment(segment, max_words_per_chunk))
        return refined

    def summarize(self, segments: list[str], *, language: str) -> str:
        del language
        return summarize_segments(segments, max_segments=2)
