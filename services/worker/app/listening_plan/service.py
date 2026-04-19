from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.context_profile import resolve_context_profile
from app.models import Chunk, ListeningPausePoint, ListeningPlanResult, RESPONSE_VERSION
from app.text_analysis import estimate_segment_load


@dataclass(slots=True)
class ListeningPlanService:
    chunking_service: ChunkingService

    def build(self, text: str, language: str = "en", target_context: str = "general") -> ListeningPlanResult:
        profile = resolve_context_profile(target_context)
        chunking = self.chunking_service.chunk_text(text=text, language=language)
        chunks = chunking.chunks
        if not chunks:
            return ListeningPlanResult(
                version=RESPONSE_VERSION,
                language=language,
                summary="",
                recommended_speed="0.85x",
                pause_points=[],
                final_pass_strategy="listen_once_without pauses after the chunk-by-chunk pass",
            )

        pause_points: list[ListeningPausePoint] = []
        running_preview: list[str] = []
        for chunk in chunks:
            running_preview.append(chunk.text)
            if not _should_pause_after(chunk, chunks):
                continue

            risk_level = _classify_pause_risk(chunk)
            pause_points.append(
                ListeningPausePoint(
                    index=len(pause_points) + 1,
                    after_chunk_order=chunk.order,
                    pause_reason=_build_pause_reason(chunk),
                    cue_en=_build_english_cue(profile.listening_focus_prompt, chunk),
                    cue_ja=_build_japanese_cue(profile.label_ja, chunk),
                    preview_text=" / ".join(running_preview[-2:]),
                    risk_level=risk_level,
                )
            )

        return ListeningPlanResult(
            version=RESPONSE_VERSION,
            language=language,
            summary=chunking.summary,
            recommended_speed=_recommend_speed(chunks, pause_points),
            pause_points=pause_points,
            final_pass_strategy=_build_final_pass_strategy(pause_points),
        )


def _should_pause_after(chunk: Chunk, chunks: list[Chunk]) -> bool:
    if chunk.order == len(chunks):
        return True
    if chunk.is_core:
        return True
    return estimate_segment_load(chunk.text) >= 7


def _classify_pause_risk(chunk: Chunk) -> str:
    score = estimate_segment_load(chunk.text)
    if score >= 9:
        return "high"
    if score >= 6:
        return "medium"
    return "low"


def _build_pause_reason(chunk: Chunk) -> str:
    if chunk.is_core:
        return "core meaning checkpoint"
    if estimate_segment_load(chunk.text) >= 7:
        return "linguistically heavy segment"
    return "light checkpoint"


def _build_english_cue(focus_prompt: str, chunk: Chunk) -> str:
    if chunk.is_core:
        return f"{focus_prompt} Pause here and confirm the main action or claim."
    return f"{focus_prompt} Pause here and keep only the key meaning before continuing."


def _build_japanese_cue(context_label: str, chunk: Chunk) -> str:
    if chunk.is_core:
        return f"{context_label}として、ここで止めて主な動きや主張だけ確認します。"
    return f"{context_label}として、ここで止めて細部より要点だけ残して次へ進みます。"


def _recommend_speed(chunks: list[Chunk], pause_points: list[ListeningPausePoint]) -> str:
    heavy_chunks = sum(1 for chunk in chunks if estimate_segment_load(chunk.text) >= 7)
    if heavy_chunks >= 2 or any(point.risk_level == "high" for point in pause_points):
        return "0.80x"
    if pause_points:
        return "0.90x"
    return "1.00x"


def _build_final_pass_strategy(pause_points: list[ListeningPausePoint]) -> str:
    if any(point.risk_level == "high" for point in pause_points):
        return "repeat the audio once more with fewer pauses after the first checkpointed pass"
    if pause_points:
        return "listen once more with pauses removed after the guided pass"
    return "listen once at natural speed after the guided preview"
