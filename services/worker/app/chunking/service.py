from __future__ import annotations

import re
from dataclasses import dataclass

from app.models import Chunk, ChunkingResult


_CLAUSE_BREAKS = re.compile(r"\s*(,|;|:|\band\b|\bbut\b|\bthat\b|\bwhich\b)\s*", re.IGNORECASE)
_VERB_HINTS = {
    "am",
    "is",
    "are",
    "was",
    "were",
    "be",
    "been",
    "being",
    "do",
    "does",
    "did",
    "have",
    "has",
    "had",
    "use",
    "uses",
    "used",
    "need",
    "needs",
    "needed",
    "make",
    "makes",
    "made",
    "propose",
    "proposes",
    "proposed",
    "show",
    "shows",
    "showed",
}


@dataclass(slots=True)
class ChunkingService:
    max_words_per_chunk: int = 6

    def chunk_text(self, text: str, language: str = "en") -> ChunkingResult:
        normalized = " ".join(text.split())
        if not normalized:
            return ChunkingResult(language=language, chunks=[], summary="")

        rough_parts = [part.strip(" ,;:") for part in _CLAUSE_BREAKS.split(normalized)]
        segments = [part for part in rough_parts if part and part not in {",", ";", ":"}]

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
                role=self._infer_role(segment, index),
                skeleton_rank=1 if self._looks_core(segment) else 2,
            )
            for index, segment in enumerate(refined)
        ]
        summary = " / ".join(chunk.text for chunk in chunks[:2])
        return ChunkingResult(language=language, chunks=chunks, summary=summary)

    def _infer_role(self, segment: str, index: int) -> str:
        lowered = segment.lower()
        if index == 0 and not self._looks_core(segment):
            return "modifier"
        if lowered.startswith(("to ", "for ", "with ", "in ", "on ", "at ", "by ")):
            return "modifier"
        if self._looks_core(segment):
            return "core"
        return "support"

    def _looks_core(self, segment: str) -> bool:
        return any(word.lower().strip(".,!?") in _VERB_HINTS for word in segment.split())
