from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any


@dataclass(slots=True)
class Chunk:
    order: int
    text: str
    role: str
    skeleton_rank: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class ChunkingResult:
    language: str
    chunks: list[Chunk]
    summary: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "language": self.language,
            "chunks": [chunk.to_dict() for chunk in self.chunks],
            "summary": self.summary,
        }


@dataclass(slots=True)
class SkeletonPart:
    order: int
    text: str
    role: str
    emphasis: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class SkeletonResult:
    language: str
    parts: list[SkeletonPart]
    summary: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "language": self.language,
            "parts": [part.to_dict() for part in self.parts],
            "summary": self.summary,
        }
