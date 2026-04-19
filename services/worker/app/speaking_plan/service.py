from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.models import RESPONSE_VERSION, SpeakingPlanResult, SpeakingStep
from app.text_analysis import estimate_segment_load


@dataclass(slots=True)
class SpeakingPlanService:
    chunking_service: ChunkingService

    def build(self, text: str, language: str = "en") -> SpeakingPlanResult:
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        if not chunks:
            return SpeakingPlanResult(
                version=RESPONSE_VERSION,
                language=language,
                summary="",
                recommended_style="short-linked-sentences",
                opener_options=[],
                bridge_phrases=[],
                steps=[],
                rescue_phrases=[],
            )

        steps = [
            SpeakingStep(
                step=index + 1,
                text=_normalize_spoken_text(chunk.text),
                purpose=_infer_speaking_purpose(chunk.role, index),
                risk_level=_classify_speaking_risk(chunk.text),
                delivery_tip_ja=_build_japanese_tip(chunk.role),
                delivery_tip_en=_build_english_tip(chunk.role),
            )
            for index, chunk in enumerate(chunks)
        ]

        return SpeakingPlanResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            recommended_style="short-linked-sentences",
            opener_options=_build_openers(chunking.summary),
            bridge_phrases=[
                "First,",
                "Next,",
                "Also,",
                "The main point is,",
            ],
            steps=steps,
            rescue_phrases=[
                "Let me say that in a shorter way.",
                "The main point is this.",
                "One moment, please.",
            ],
        )


def _normalize_spoken_text(text: str) -> str:
    stripped = text.strip()
    if not stripped.endswith("."):
        return f"{stripped}."
    return stripped


def _infer_speaking_purpose(role: str, index: int) -> str:
    if index == 0:
        return "opener"
    if role == "core":
        return "main_point"
    if role == "modifier":
        return "context"
    return "support"


def _classify_speaking_risk(text: str) -> str:
    score = estimate_segment_load(text)
    if score >= 9:
        return "high"
    if score >= 6:
        return "medium"
    return "low"


def _build_japanese_tip(role: str) -> str:
    if role == "core":
        return "この文だけでも伝わるので、ここを先に安定して言います。"
    if role == "modifier":
        return "背景説明なので、苦しければ短くして大丈夫です。"
    return "一息で言える長さを優先します。"


def _build_english_tip(role: str) -> str:
    if role == "core":
        return "Say this first even if you need to stop after it."
    if role == "modifier":
        return "This is background information, so you can shorten it if needed."
    return "Keep it short enough to say in one breath."


def _build_openers(summary: str) -> list[str]:
    if not summary:
        return ["The main point is this."]
    return [
        f"The main point is: {summary}.",
        f"In short, {summary}.",
    ]
