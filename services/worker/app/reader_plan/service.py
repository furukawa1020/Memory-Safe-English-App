from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.models import (
    RESPONSE_VERSION,
    Chunk,
    CollapsedChunk,
    ReaderFocusStep,
    ReaderPlanResult,
)


@dataclass(slots=True)
class ReaderPlanService:
    chunking_service: ChunkingService

    def build(self, text: str, language: str = "en") -> ReaderPlanResult:
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        if not chunks:
            return ReaderPlanResult(
                version=RESPONSE_VERSION,
                language=language,
                summary="",
                recommended_mode="progressive",
                focus_steps=[],
                collapsed_chunks=[],
            )

        focus_chunks = [chunk for chunk in chunks if chunk.is_core]
        if not focus_chunks:
            focus_chunks = chunks

        focus_steps: list[ReaderFocusStep] = []
        collapsed_chunks: list[CollapsedChunk] = []

        for step_index, focus_chunk in enumerate(focus_chunks, start=1):
            before_chunks, after_chunks = _collect_support_chunks(chunks, focus_chunks, focus_chunk)
            focus_steps.append(
                ReaderFocusStep(
                    step=step_index,
                    chunk_order=focus_chunk.order,
                    text=focus_chunk.text,
                    role=focus_chunk.role,
                    support_before=[chunk.text for chunk in before_chunks],
                    support_after=[chunk.text for chunk in after_chunks],
                    guidance_ja=_build_japanese_guidance(focus_chunk, before_chunks, after_chunks),
                    guidance_en=_build_english_guidance(focus_chunk, before_chunks, after_chunks),
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

        return ReaderPlanResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            recommended_mode="progressive",
            focus_steps=focus_steps,
            collapsed_chunks=collapsed_chunks,
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


def _build_japanese_guidance(focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    parts = [f"まず「{focus_chunk.text}」を主軸として読みます。"]
    if before_chunks:
        parts.append("前の補助情報は後回しにして大丈夫です。")
    if after_chunks:
        parts.append("後ろの補足は主軸をつかんでから追加します。")
    if focus_chunk.role == "core":
        parts.append("動きや主張を先に確定してください。")
    return " ".join(parts)


def _build_english_guidance(focus_chunk: Chunk, before_chunks: list[Chunk], after_chunks: list[Chunk]) -> str:
    parts = [f"Start with '{focus_chunk.text}' as the main idea."]
    if before_chunks:
        parts.append("You can delay the earlier support details.")
    if after_chunks:
        parts.append("Add the following support after the core idea is stable.")
    if focus_chunk.role == "core":
        parts.append("Lock in the action or claim first.")
    return " ".join(parts)
