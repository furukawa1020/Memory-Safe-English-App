from __future__ import annotations

from dataclasses import dataclass

from app.chunking import ChunkingService
from app.config import Settings


@dataclass(slots=True)
class Application:
    settings: Settings
    chunking_service: ChunkingService


def build_application(settings: Settings | None = None) -> Application:
    resolved_settings = settings or Settings.load()
    return Application(
        settings=resolved_settings,
        chunking_service=ChunkingService(max_words_per_chunk=resolved_settings.max_words_per_chunk),
    )
