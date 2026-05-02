from __future__ import annotations

from dataclasses import dataclass

from app.analytics_summary import AnalyticsSummaryService
from app.assessment import AssessmentService
from app.analysis import AnalysisService
from app.collapse_patterns import CollapsePatternService
from app.chunking import ChunkingService
from app.config import Settings
from app.listening_plan import ListeningPlanService
from app.nlp_backend import build_chunk_backend
from app.practice_set import PracticeSetService
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
    collapse_pattern_service: CollapsePatternService
    analytics_summary_service: AnalyticsSummaryService
    practice_set_service: PracticeSetService
    analysis_service: AnalysisService
    rate_limiter: SlidingWindowRateLimiter


def build_application(settings: Settings | None = None) -> Application:
    resolved_settings = settings or Settings.load()
    resolved_settings.validate()
    chunking_service = ChunkingService(
        max_words_per_chunk=resolved_settings.max_words_per_chunk,
        backend=build_chunk_backend(resolved_settings),
    )
    skeleton_service = SkeletonService()
    reader_plan_service = ReaderPlanService(chunking_service=chunking_service)
    listening_plan_service = ListeningPlanService(chunking_service=chunking_service)
    speaking_plan_service = SpeakingPlanService(chunking_service=chunking_service)
    rescue_plan_service = RescuePlanService(chunking_service=chunking_service)
    assessment_service = AssessmentService()
    collapse_pattern_service = CollapsePatternService(chunking_service=chunking_service)
    analytics_summary_service = AnalyticsSummaryService(
        assessment_service=assessment_service,
        collapse_pattern_service=collapse_pattern_service,
    )
    practice_set_service = PracticeSetService(
        reader_plan_service=reader_plan_service,
        listening_plan_service=listening_plan_service,
        speaking_plan_service=speaking_plan_service,
        rescue_plan_service=rescue_plan_service,
        assessment_service=assessment_service,
        collapse_pattern_service=collapse_pattern_service,
    )
    return Application(
        settings=resolved_settings,
        chunking_service=chunking_service,
        skeleton_service=skeleton_service,
        reader_plan_service=reader_plan_service,
        listening_plan_service=listening_plan_service,
        speaking_plan_service=speaking_plan_service,
        rescue_plan_service=rescue_plan_service,
        assessment_service=assessment_service,
        collapse_pattern_service=collapse_pattern_service,
        analytics_summary_service=analytics_summary_service,
        practice_set_service=practice_set_service,
        analysis_service=AnalysisService(
            chunk_analyzer=chunking_service,
            skeleton_analyzer=skeleton_service,
            reader_plan_analyzer=reader_plan_service,
            listening_plan_analyzer=listening_plan_service,
            speaking_plan_analyzer=speaking_plan_service,
            rescue_plan_analyzer=rescue_plan_service,
            assessment_analyzer=assessment_service,
            collapse_pattern_analyzer=collapse_pattern_service,
            analytics_summary_analyzer=analytics_summary_service,
            practice_set_analyzer=practice_set_service,
        ),
        rate_limiter=SlidingWindowRateLimiter(
            max_requests=resolved_settings.rate_limit_max_requests,
            window_seconds=resolved_settings.rate_limit_window_seconds,
        ),
    )
