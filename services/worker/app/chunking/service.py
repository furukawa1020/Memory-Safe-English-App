from __future__ import annotations

from dataclasses import dataclass

from app.models import RESPONSE_VERSION, Chunk, ChunkingResult
from app.text_analysis import infer_role, looks_core, normalize_text, segment_text, split_long_segment, summarize_segments


@dataclass(slots=True)
class ChunkingService:
    max_words_per_chunk: int = 6

    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult:
        normalized = normalize_text(text)
        if not normalized:
            return ChunkingResult(version=RESPONSE_VERSION, language=language, chunks=[], summary="")

        segments = segment_text(normalized)
        if not segments:
            segments = [normalized]

        refined: list[str] = []
        for segment in segments:
            refined.extend(split_long_segment(segment, self.max_words_per_chunk))

        chunks = [
            Chunk(
                order=index + 1,
                text=segment,
                role=infer_role(segment, index),
                skeleton_rank=1 if looks_core(segment) else 2,
            )
            for index, segment in enumerate(refined)
        ]
        summary_segments = [chunk.text for chunk in chunks if chunk.role == "core"] or [chunk.text for chunk in chunks]
        summary = summarize_segments(summary_segments, max_segments=2)
        return ChunkingResult(version=RESPONSE_VERSION, language=language, chunks=chunks, summary=summary)
