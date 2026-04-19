from __future__ import annotations

from dataclasses import dataclass

from app.assessment import AssessmentService
from app.analysis import AnalysisService
from app.chunking import ChunkingService
from app.config import Settings
from app.listening_plan import ListeningPlanService
from app.reader_plan import ReaderPlanService
from app.rate_limit import SlidingWindowRateLimiter
from app.rescue_plan import RescuePlanService
from app.speaking_plan import SpeakingPlanService
from app.skeleton import SkeletonService


@dataclass(slots=True)
class Application:
    settings: Settings
    chunking_service: ChunkingService
    skeleton_service: SkeletonService
    reader_plan_service: ReaderPlanService
    listening_plan_service: ListeningPlanService
    speaking_plan_service: SpeakingPlanService
    rescue_plan_service: RescuePlanService
    assessment_service: AssessmentService
    analysis_service: AnalysisService
    rate_limiter: SlidingWindowRateLimiter


def build_application(settings: Settings | None = None) -> Application:
    resolved_settings = settings or Settings.load()
    resolved_settings.validate()
    chunking_service = ChunkingService(max_words_per_chunk=resolved_settings.max_words_per_chunk)
    skeleton_service = SkeletonService()
    reader_plan_service = ReaderPlanService(chunking_service=chunking_service)
    listening_plan_service = ListeningPlanService(chunking_service=chunking_service)
    speaking_plan_service = SpeakingPlanService(chunking_service=chunking_service)
    rescue_plan_service = RescuePlanService(chunking_service=chunking_service)
    assessment_service = AssessmentService()
    return Application(
        settings=resolved_settings,
        chunking_service=chunking_service,
        skeleton_service=skeleton_service,
        reader_plan_service=reader_plan_service,
        listening_plan_service=listening_plan_service,
        speaking_plan_service=speaking_plan_service,
        rescue_plan_service=rescue_plan_service,
        assessment_service=assessment_service,
        analysis_service=AnalysisService(
            chunk_analyzer=chunking_service,
            skeleton_analyzer=skeleton_service,
            reader_plan_analyzer=reader_plan_service,
            listening_plan_analyzer=listening_plan_service,
            speaking_plan_analyzer=speaking_plan_service,
            rescue_plan_analyzer=rescue_plan_service,
            assessment_analyzer=assessment_service,
        ),
        rate_limiter=SlidingWindowRateLimiter(
            max_requests=resolved_settings.rate_limit_max_requests,
            window_seconds=resolved_settings.rate_limit_window_seconds,
        ),
    )
