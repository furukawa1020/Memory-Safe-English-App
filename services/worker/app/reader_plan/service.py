from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.context_profile import resolve_context_profile
from app.models import (
    RESPONSE_VERSION,
    Chunk,
    CollapsedChunk,
    ReaderFocusStep,
    ReaderHotspot,
    ReaderPlanResult,
)
from app.text_analysis import classify_density, estimate_segment_load


@dataclass(slots=True)
class ReaderPlanService:
    chunking_service: ChunkingService

    def build(self, text: str, language: str = "en", target_context: str = "general") -> ReaderPlanResult:
        profile = resolve_context_profile(target_context)
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        if not chunks:
            return ReaderPlanResult(
                version=RESPONSE_VERSION,
                language=language,
                summary="",
                recommended_mode="progressive",
                display_strategy="step_by_step",
                focus_steps=[],
                collapsed_chunks=[],
                hotspots=[],
            )

        focus_chunks = [chunk for chunk in chunks if chunk.is_core]
        if not focus_chunks:
            focus_chunks = chunks

        focus_steps: list[ReaderFocusStep] = []
        collapsed_chunks: list[CollapsedChunk] = []
        hotspots: list[ReaderHotspot] = []

        for step_index, focus_chunk in enumerate(focus_chunks, start=1):
            before_chunks, after_chunks = _collect_support_chunks(chunks, focus_chunks, focus_chunk)
            support_density = classify_density(len(before_chunks), len(after_chunks))
            overload_risk = _classify_overload_risk(focus_chunk, before_chunks, after_chunks)
            presentation_hint = _build_presentation_hint(overload_risk, support_density)

            focus_steps.append(
                ReaderFocusStep(
                    step=step_index,
                    chunk_order=focus_chunk.order,
                    text=focus_chunk.text,
                    role=focus_chunk.role,
                    support_before=[chunk.text for chunk in before_chunks],
                    support_after=[chunk.text for chunk in after_chunks],
                    support_density=support_density,
                    overload_risk=overload_risk,
                    presentation_hint=presentation_hint,
                    guidance_ja=_build_japanese_guidance(profile.label_ja, focus_chunk, before_chunks, after_chunks),
                    guidance_en=_build_english_guidance(profile.listening_focus_prompt, focus_chunk, before_chunks, after_chunks),
                )
            )

            collapsed_chunks.extend(
                CollapsedChunk(
                    chunk_order=chunk.order,
                    text=chunk.text,
                    role=chunk.role,
                    anchor_step=step_index,
                    placement="before",
                )
                for chunk in before_chunks
            )
            collapsed_chunks.extend(
                CollapsedChunk(
                    chunk_order=chunk.order,
                    text=chunk.text,
                    role=chunk.role,
                    anchor_step=step_index,
                    placement="after",
                )
                for chunk in after_chunks
            )

            if overload_risk in {"medium", "high"}:
                hotspots.append(
                    ReaderHotspot(
                        chunk_order=focus_chunk.order,
                        text=focus_chunk.text,
                        risk_level=overload_risk,
                        reason=_build_hotspot_reason(focus_chunk, before_chunks, after_chunks),
                        recommendation=presentation_hint,
                    )
                )

        return ReaderPlanResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            recommended_mode="progressive",
            display_strategy=_recommend_display_strategy(focus_steps),
            focus_steps=focus_steps,
            collapsed_chunks=collapsed_chunks,
            hotspots=hotspots,
        )


def _collect_support_chunks(
    chunks: list[Chunk],
    focus_chunks: list[Chunk],
    focus_chunk: Chunk,
) -> tuple[list[Chunk], list[Chunk]]:
    focus_orders = {chunk.order for chunk in focus_chunks}
    focus_index = next(index for index, chunk in enumerate(focus_chunks) if chunk.order == focus_chunk.order)
    previous_focus_order = focus_chunks[focus_index - 1].order if focus_index > 0 else 0
    next_focus_order = focus_chunks[focus_index + 1].order if focus_index + 1 < len(focus_chunks) else 10**9

    before = [
        chunk
        for chunk in chunks
        if previous_focus_order < chunk.order < focus_chunk.order and chunk.order not in focus_orders
    ]
    after = [
        chunk
        for chunk in chunks
        if focus_chunk.order < chunk.order < next_focus_order and chunk.order not in focus_orders
    ]
    return before, after


def _build_japanese_guidance(context_label: str, focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    parts = [f"{context_label}として、まず「{focus_chunk.text}」を主軸として読みます。"]
    if before_chunks:
        parts.append("前の補助情報は後回しにして大丈夫です。")
    if after_chunks:
        parts.append("後ろの補足は主軸をつかんでから追加します。")
    if focus_chunk.role == "core":
        parts.append("動きや主張を先に確定してください。")
    return " ".join(parts)


def _build_english_guidance(focus_prompt: str, focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    parts = [focus_prompt, f"Start with '{focus_chunk.text}' as the main idea."]
    if before_chunks:
        parts.append("You can delay the earlier support details.")
    if after_chunks:
        parts.append("Add the following support after the core idea is stable.")
    if focus_chunk.role == "core":
        parts.append("Lock in the action or claim first.")
    return " ".join(parts)


def _classify_overload_risk(focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    score = estimate_segment_load(focus_chunk.text)
    score += len(before_chunks) * 2
    score += len(after_chunks) * 2
    if score >= 11:
        return "high"
    if score >= 7:
        return "medium"
    return "low"


def _build_presentation_hint(overload_risk: str, support_density: str) -> str:
    if overload_risk == "high":
        return "show only the focus chunk first, then reveal one support chunk at a time"
    if support_density == "dense":
        return "keep support collapsed until the main chunk is confirmed"
    if overload_risk == "medium":
        return "dim support chunks and keep the core chunk visually anchored"
    return "show the focus chunk with nearby support visible"


def _build_hotspot_reason(focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    reasons: list[str] = []
    if estimate_segment_load(focus_chunk.text) >= 7:
        reasons.append("the focus chunk is linguistically heavy")
    if before_chunks:
        reasons.append("support appears before the main chunk")
    if len(after_chunks) >= 2:
        reasons.append("multiple support chunks follow the main chunk")
    if not reasons:
        reasons.append("the step may still require staged reading")
    return "; ".join(reasons)


def _recommend_display_strategy(focus_steps: list[ReaderFocusStep]) -> str:
    if any(step.overload_risk == "high" for step in focus_steps):
        return "focus-first"
    if any(step.support_density == "dense" for step in focus_steps):
        return "assisted"
    return "chunk"
