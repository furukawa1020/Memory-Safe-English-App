from __future__ import annotations

from app.config import Settings
from app.nlp_backend.heuristic import HeuristicChunkBackend
from app.nlp_backend.protocols import ChunkBackend
from app.nlp_backend.transformer import TransformerChunkBackend


def build_chunk_backend(settings: Settings) -> ChunkBackend:
    if settings.nlp_backend == "transformer":
        return TransformerChunkBackend(
            model_name=settings.transformer_model_name,
            task=settings.transformer_task,
            device=settings.transformer_device,
            max_new_tokens=settings.transformer_max_new_tokens,
        )
    return HeuristicChunkBackend()
