from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Any

RESPONSE_VERSION = "2026-04-19"


@dataclass(slots=True)
class Chunk:
    order: int
    text: str
    role: str
    skeleton_rank: int

    @property
    def is_core(self) -> bool:
        return self.skeleton_rank == 1 or self.role == "core"

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class ChunkingResult:
    version: str
    language: str
    chunks: list[Chunk]
    summary: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
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
    version: str
    language: str
    parts: list[SkeletonPart]
    summary: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "parts": [part.to_dict() for part in self.parts],
            "summary": self.summary,
        }


@dataclass(slots=True)
class ReaderFocusStep:
    step: int
    chunk_order: int
    text: str
    role: str
    support_before: list[str]
    support_after: list[str]
    support_density: str
    overload_risk: str
    presentation_hint: str
    guidance_ja: str
    guidance_en: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class CollapsedChunk:
    chunk_order: int
    text: str
    role: str
    anchor_step: int
    placement: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class ReaderHotspot:
    chunk_order: int
    text: str
    risk_level: str
    reason: str
    recommendation: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class ReaderPlanResult:
    version: str
    language: str
    summary: str
    recommended_mode: str
    display_strategy: str
    focus_steps: list[ReaderFocusStep]
    collapsed_chunks: list[CollapsedChunk]
    hotspots: list[ReaderHotspot]

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "summary": self.summary,
            "recommended_mode": self.recommended_mode,
            "display_strategy": self.display_strategy,
            "focus_steps": [step.to_dict() for step in self.focus_steps],
            "collapsed_chunks": [chunk.to_dict() for chunk in self.collapsed_chunks],
            "hotspots": [hotspot.to_dict() for hotspot in self.hotspots],
        }


@dataclass(slots=True)
class ListeningPausePoint:
    index: int
    after_chunk_order: int
    pause_reason: str
    cue_en: str
    cue_ja: str
    preview_text: str
    risk_level: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class ListeningPlanResult:
    version: str
    language: str
    summary: str
    recommended_speed: str
    pause_points: list[ListeningPausePoint]
    final_pass_strategy: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "summary": self.summary,
            "recommended_speed": self.recommended_speed,
            "pause_points": [pause_point.to_dict() for pause_point in self.pause_points],
            "final_pass_strategy": self.final_pass_strategy,
        }
