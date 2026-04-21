from __future__ import annotations

from dataclasses import dataclass

from app.models import RESPONSE_VERSION, Chunk, ChunkingResult
from app.nlp_backend.protocols import ChunkBackend
from app.text_analysis import infer_role, looks_core, normalize_text, summarize_segments


@dataclass(slots=True)
class ChunkingService:
    max_words_per_chunk: int = 6
    backend: ChunkBackend | None = None

    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult:
        normalized = normalize_text(text)
        if not normalized:
            return ChunkingResult(version=RESPONSE_VERSION, language=language, chunks=[], summary="")

        backend = self.backend
        if backend is None:
            from app.nlp_backend.heuristic import HeuristicChunkBackend

            backend = HeuristicChunkBackend()

        refined = backend.chunk(normalized, language=language, max_words_per_chunk=self.max_words_per_chunk)
        if not refined:
            refined = [normalized]

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
        summary = backend.summarize(summary_segments, language=language) or summarize_segments(summary_segments, max_segments=2)
        return ChunkingResult(version=RESPONSE_VERSION, language=language, chunks=chunks, summary=summary)
