from __future__ import annotations

from dataclasses import dataclass

from app.models import Chunk, ChunkingResult
from app.text_analysis import infer_role, looks_core, normalize_text, segment_text


@dataclass(slots=True)
class ChunkingService:
    max_words_per_chunk: int = 6

    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult:
        normalized = normalize_text(text)
        if not normalized:
            return ChunkingResult(language=language, chunks=[], summary="")

        segments = segment_text(normalized)

        refined: list[str] = []
        for segment in segments:
            words = segment.split()
            if len(words) <= self.max_words_per_chunk:
                refined.append(segment)
                continue

            start = 0
            while start < len(words):
                refined.append(" ".join(words[start : start + self.max_words_per_chunk]))
                start += self.max_words_per_chunk

        chunks = [
            Chunk(
                order=index + 1,
                text=segment,
                role=infer_role(segment, index),
                skeleton_rank=1 if looks_core(segment) else 2,
            )
            for index, segment in enumerate(refined)
        ]
        summary = " / ".join(chunk.text for chunk in chunks[:2])
        return ChunkingResult(language=language, chunks=chunks, summary=summary)
