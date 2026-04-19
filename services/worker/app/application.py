from __future__ import annotations

from dataclasses import dataclass

from app.analysis import AnalysisService
from app.chunking import ChunkingService
from app.config import Settings
from app.rate_limit import SlidingWindowRateLimiter
from app.skeleton import SkeletonService


@dataclass(slots=True)
class Application:
    settings: Settings
    chunking_service: ChunkingService
    skeleton_service: SkeletonService
    analysis_service: AnalysisService
    rate_limiter: SlidingWindowRateLimiter


def build_application(settings: Settings | None = None) -> Application:
    resolved_settings = settings or Settings.load()
    resolved_settings.validate()
    chunking_service = ChunkingService(max_words_per_chunk=resolved_settings.max_words_per_chunk)
    skeleton_service = SkeletonService()
    return Application(
        settings=resolved_settings,
        chunking_service=chunking_service,
        skeleton_service=skeleton_service,
        analysis_service=AnalysisService(
            chunk_analyzer=chunking_service,
            skeleton_analyzer=skeleton_service,
        ),
        rate_limiter=SlidingWindowRateLimiter(
            max_requests=resolved_settings.rate_limit_max_requests,
            window_seconds=resolved_settings.rate_limit_window_seconds,
        ),
    )
