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


@dataclass(slots=True)
class SpeakingStep:
    step: int
    text: str
    purpose: str
    risk_level: str
    delivery_tip_ja: str
    delivery_tip_en: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class SpeakingPlanResult:
    version: str
    language: str
    summary: str
    recommended_style: str
    opener_options: list[str]
    bridge_phrases: list[str]
    steps: list[SpeakingStep]
    rescue_phrases: list[str]

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "summary": self.summary,
            "recommended_style": self.recommended_style,
            "opener_options": self.opener_options,
            "bridge_phrases": self.bridge_phrases,
            "steps": [step.to_dict() for step in self.steps],
            "rescue_phrases": self.rescue_phrases,
        }


@dataclass(slots=True)
class RescuePhrase:
    category: str
    phrase_en: str
    phrase_ja: str
    use_when: str
    priority: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class RescuePlanResult:
    version: str
    language: str
    summary: str
    overload_level: str
    primary_strategy: str
    phrases: list[RescuePhrase]

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "summary": self.summary,
            "overload_level": self.overload_level,
            "primary_strategy": self.primary_strategy,
            "phrases": [phrase.to_dict() for phrase in self.phrases],
        }


@dataclass(slots=True)
class AssessmentProfileResult:
    version: str
    language: str
    target_context: str
    profile_label: str
    notice: str
    reading_load_score: int
    listening_load_score: int
    speaking_load_score: int
    recommended_reader_mode: str
    recommended_listening_mode: str
    recommended_speaking_mode: str
    recommended_display_density: str
    recommended_pause_frequency: str
    reasons: list[str]

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class CollapseSite:
    chunk_order: int
    text: str
    role: str
    risk_level: str
    stop_count: int
    reasons: list[str]
    recommendation: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class CollapsePatternResult:
    version: str
    language: str
    summary: str
    dominant_pattern: str
    sites: list[CollapseSite]

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "summary": self.summary,
            "dominant_pattern": self.dominant_pattern,
            "sites": [site.to_dict() for site in self.sites],
        }


@dataclass(slots=True)
class PracticeRecommendation:
    area: str
    title: str
    reason: str
    priority: int

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass(slots=True)
class AnalyticsSummaryResult:
    version: str
    language: str
    target_context: str
    next_focus: str
    assessment: AssessmentProfileResult
    collapse_patterns: CollapsePatternResult
    recommendations: list[PracticeRecommendation]

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": self.version,
            "language": self.language,
            "target_context": self.target_context,
            "next_focus": self.next_focus,
            "assessment": self.assessment.to_dict(),
            "collapse_patterns": self.collapse_patterns.to_dict(),
            "recommendations": [item.to_dict() for item in self.recommendations],
        }
